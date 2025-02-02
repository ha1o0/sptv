<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref, reactive } from "vue";
import { useRouter } from "vue-router";
import {
  EyeTwoTone,
  EditTwoTone,
  DeleteTwoTone,
  InteractionTwoTone,
  PlusCircleTwoTone,
} from "@ant-design/icons-vue";
import Breadcrumb from "@/components/Breadcrumb.vue";
import { convertTimestampsDeep } from "@/utils/common.ts";
import { Form } from "ant-design-vue";
import { M3UParser } from "@/utils/m3u-parser";
import { message } from "ant-design-vue";
import { proxyPrefix } from "@/utils/video";

// 获取路由对象
const router = useRouter();

// 面包屑数据
const breadcrumbItems = ref([
  { name: "配置列表", path: "/" }, // 当前页不需要跳转
]);

// 表格列定义
const columns = [
  {
    title: "名称",
    dataIndex: "name",
    key: "name",
  },
  {
    title: "地址",
    dataIndex: "url",
    key: "url",
  },
  {
    title: "更新时间",
    dataIndex: "updated_at",
    key: "updated_at",
  },
  {
    title: "操作",
    key: "action",
  },
];

// 表格数据
const data = ref([]);

// Modal 弹框状态
const visible = ref(false);
const isUpdate = ref(false);

// 打开 Modal
const showModal = () => {
  visible.value = true;
};

const messageDuration = 2.5;

interface FormState {
  id: number;
  name: string;
  url: string;
}

let formState = reactive<FormState>({
  id: -1,
  name: "",
  url: "",
});

const { resetFields } = Form.useForm(formState);

const onFinish = async () => {
  if (isUpdate.value) {
    await updateSource();
  } else {
    await addSource();
  }
  message.success("操作成功", messageDuration);
  getSources();
};

const onFinishFailed = (errorInfo: any) => {
  console.log("Failed:", errorInfo);
};

async function addSource() {
  await invoke("add_video_source_command", formState);
}

async function deleteSource(id: any) {
  await invoke("delete_video_source_command", { id });
}

async function updateSource() {
  await invoke("update_video_source_command", formState);
}

async function addVideoUrls(sourceId: any, videoUrls: any) {
  await invoke("add_video_urls_command", { sourceId, videoUrls });
}

async function getSources() {
  const sources: any = await invoke("get_video_sources_command");
  console.log("sources: ", sources);
  sources.forEach((source: any) => {
    source.key = source.id.toString();
  });
  data.value = convertTimestampsDeep(sources);
  visible.value = false;
}

const toAddConfig = () => {
  resetFields();
  isUpdate.value = false;
  showModal();
};

const fetchM3U = async (url: string) => {
  try {
    const response = await fetch(`${proxyPrefix}${encodeURIComponent(url)}`);
    if (!response.ok) throw new Error("网络请求失败");

    const m3uContent = await response.text();
    return m3uContent;
  } catch (error) {
    message.error("获取m3u内容失败", messageDuration);
  }
};

// 跳转到配置详情页
const toConfigDetail = (key: string, name: string) => {
  console.log("toConfigDetail: ", key);
  router.push({ name: "ConfigDetail", params: { id: key }, query: { configName: name } });
};

const toUpdateVideoUrls = (updatedFormState: FormState) => {
  try {
    message
      .loading("正在更新频道列表")
      .then(() => updateVideoUrls(updatedFormState))
      .then(() => getSources())
      .then(() => message.success("更新频道列表完成", messageDuration));
  } catch (error) {
    message.error("更新频道列表失败, 请重试", messageDuration);
  }
};

// 更新配置中的视频地址
const updateVideoUrls = async (updatedFormState: FormState) => {
  console.log("toUpdateVideoUrls: ", updatedFormState);
  const { id, url } = updatedFormState;
  const m3uContent = await fetchM3U(url);
  // console.log("content: ", m3uContent);
  if (!m3uContent) return;
  const config = {
    nameFields: ["tvg-name", "tv-name", "name"],
    logoFields: ["tvg-logo"],
    groupFields: ["group-title", "group"],
  };

  // 解析结果
  const result = M3UParser.parse(m3uContent, config);
  const videoUrls = M3UParser.flattenChannels(result, +id);
  console.log(videoUrls);
  await addVideoUrls(+id, videoUrls);
};

// 更新配置
const toUpdateConfig = (updatedFormState: FormState) => {
  isUpdate.value = true;
  formState.id = updatedFormState.id;
  formState.name = updatedFormState.name;
  formState.url = updatedFormState.url;
  showModal();
};

// 删除配置
const toDeleteConfig = async (key: string) => {
  await deleteSource(+key);
  message.success("删除成功", messageDuration);
  await getSources();
};

getSources();
</script>

<template>
  <div>
    <div class="page-header">
      <Breadcrumb :items="breadcrumbItems" />
      <a-button type="primary" shape="round" @click="toAddConfig">
        <template #icon>
          <PlusCircleTwoTone />
        </template>
        添加配置
      </a-button>
    </div>

    <!-- 表格 -->
    <a-table :columns="columns" :data-source="data">
      <template #headerCell="{ column }">
        <template v-if="column.key === 'name'">
          <span>名称</span>
        </template>
      </template>

      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'action'">
          <span>
            <a alt="查看频道" @click="toConfigDetail(record.key, record.name)">
              <EyeTwoTone />
            </a>
            <a-divider type="vertical" />
            <a-popconfirm
              title="确定更新该配置频道列表吗？"
              ok-text="是"
              cancel-text="否"
              @confirm="toUpdateVideoUrls(record)"
            >
              <a alt="更新频道"><InteractionTwoTone /></a>
            </a-popconfirm>
            <a-divider type="vertical" />
            <a alt="编辑" @click="toUpdateConfig(record)"><EditTwoTone /></a>
            <a-divider type="vertical" />
            <a-popconfirm
              title="确定删除该配置吗？"
              ok-text="是"
              cancel-text="否"
              @confirm="toDeleteConfig(record.key)"
            >
              <a alt="删除"><DeleteTwoTone two-tone-color="#eb2f96" /></a>
            </a-popconfirm>
          </span>
        </template>
      </template>

      <template #emptyText>
        <a-empty description="暂无数据" />
      </template>
    </a-table>

    <!-- Modal 弹框 -->
    <a-modal
      v-model:open="visible"
      :title="isUpdate ? '更新配置' : '添加配置'"
      :footer="null"
    >
      <a-form
        :model="formState"
        name="basic"
        autocomplete="off"
        @finish="onFinish"
        @finishFailed="onFinishFailed"
      >
        <a-form-item
          label="名称"
          name="name"
          :rules="[{ required: true, message: '请输入配置名称' }]"
        >
          <a-input v-model:value="formState.name" />
        </a-form-item>

        <a-form-item
          label="链接"
          name="url"
          :rules="[{ required: true, message: '请输入配置链接' }]"
        >
          <a-input v-model:value="formState.url" />
        </a-form-item>

        <a-form-item>
          <a-button type="primary" html-type="submit">确定</a-button>
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<style scoped lang="scss">
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
