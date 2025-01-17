<script setup lang="ts">
import { M3UParser, M3UGroup } from "@/utils/m3u-parser";
import { ref } from 'vue';
const playlist = ref<M3UGroup[]>([]);

const activeKey = ref('');

// 示例 M3U 文件内容
const m3uContent = `
#EXTM3U x-tvg-url="https://epg.aptv.app/pp.xml.gz"
#EXT-X-APP APTV
#EXT-X-APTV-TYPE remote
#EXT-X-SUB-URL https://gh.aptv.app/https://raw.githubusercontent.com/Kimentanm/aptv/master/m3u/iptv.m3u

#EXTINF:-1 tvg-id="重温经典" tvg-name="重温经典" group-title="央视IPV4",重温经典
http://148.135.93.213:81/live.php?id=重温经典
#EXTINF:-1 tvg-name="CHC家庭影院" group-title="央视IPV4",CHC家庭影院
http://148.135.93.213:81/live.php?id=CHC家庭影院
#EXTINF:-1 tvg-name="CHC动作电影" group-title="央视IPV4",CHC动作电影
http://148.135.93.213:81/live.php?id=CHC动作电影
#EXTINF:-1 tvg-id="CCTV1" tvg-name="CCTV1" group-title="央视IPV4",CCTV1
http://148.135.93.213:81/live.php?id=CCTV1
#EXTINF:-1 tvg-id="CCTV2" tvg-name="CCTV2" group-title="央视IPV4",CCTV2
http://148.135.93.213:81/live.php?id=CCTV2
#EXTINF:-1 tvg-id="CCTV3" tvg-name="CCTV3" group-title="央视IPV4",CCTV3
http://148.135.93.213:81/live.php?id=CCTV3
#EXTINF:-1 tvg-id="CCTV4" tvg-name="CCTV4" group-title="央视IPV4",CCTV4
http://148.135.93.213:81/live.php?id=CCTV4
#EXTINF:-1 tvg-id="CCTV5" tvg-name="CCTV5" group-title="央视IPV4",CCTV5
http://148.135.93.213:81/live.php?id=CCTV5
#EXTINF:-1 tvg-id="CCTV5PLUS" tvg-name="CCTV5+" group-title="央视IPV4",CCTV5+
http://148.135.93.213:81/live.php?id=CCTV5p
#EXTINF:-1 tvg-id="CCTV6" tvg-name="CCTV6" group-title="央视IPV4",CCTV6
http://148.135.93.213:81/live.php?id=CCTV6
#EXTINF:-1 tvg-id="CCTV7" tvg-name="CCTV7" group-title="央视IPV4",CCTV7

#EXTINF:-1 tvg-id="" tvg-name="五星体育" group-title="卫视IPV4",五星体育
http://148.135.93.213:81/live.php?id=五星体育
#EXTINF:-1 tvg-id="凤凰资讯" tvg-name="凤凰资讯" http-user-agent="iPhone" group-title="卫视IPV4",凤凰资讯
http://148.135.93.213:81/phenix.php?id=fhzx
#EXTINF:-1 tvg-id="凤凰中文" tvg-name="凤凰中文" http-user-agent="iPhone" group-title="卫视IPV4",凤凰中文
http://148.135.93.213:81/phenix.php?id=fhzw
#EXTINF:-1 tvg-id="凤凰香港" tvg-name="凤凰香港" http-user-agent="iPhone" group-title="卫视IPV4",凤凰香港
http://148.135.93.213:81/phenix.php?id=fhhk
#EXTINF:-1 tvg-id="北京卫视" tvg-name="北京卫视" group-title="卫视IPV4",北京卫视
http://148.135.93.213:81/live.php?id=北京卫视
`;

const config = {
  nameFields: ["tvg-name", "tv-name", "name"],
  logoFields: ["tvg-logo"],
  groupFields: ["group-title", "group"],
};

// 解析结果
const result = M3UParser.parse(m3uContent, config);
console.log(result);
playlist.value = result;
activeKey.value = result[0].groupName;
console.log(playlist.value);

</script>

<template>
    <div>
      <a-tabs v-model:activeKey="activeKey" type="editable-card" hideAdd>
        <a-tab-pane v-for="item in playlist" :key="item.groupName" :tab="item.groupName" :closable="false">
          <a-flex wrap="wrap" gap="small">
            <div class="channel-card" v-for="channel in item.channels" :key="channel" type="primary">
              {{ channel.name }} - {{ channel.url }}
            </div>
          </a-flex>
        </a-tab-pane>
      </a-tabs>
    </div>
</template>

<style lang="scss" scoped>

.ant-tabs-tab {
  border-radius: 32px;
}

.channel-card {
  display: flex;
  width: 200px;
  height: 160px;
  background: lightblue;
  word-break: break-all;
}

</style>
