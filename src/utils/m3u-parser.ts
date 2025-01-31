export interface M3UChannel {
  url: string; // 频道链接
  name?: string; // 频道名称
  logo?: string; // 频道 logo
  urls?: Array<string>; // 频道多个源
}

export interface M3UGroup {
  groupName: string; // 分组名称
  channels: M3UChannel[]; // 该分组下的频道列表
}

export interface ParseConfig {
  nameFields: string[]; // 用于匹配频道名字的字段
  logoFields: string[]; // 用于匹配频道 logo 的字段
  groupFields: string[]; // 用于匹配分组的字段
}

const DEFAULT_CONFIG: ParseConfig = {
  nameFields: ["tvg-name", "tv-name", "name"], // 支持多个字段名
  logoFields: ["tvg-logo", "logo"],
  groupFields: ["group-title", "group"],
};

export class M3UParser {
  /**
   * 解析 M3U 内容
   * @param m3uContent M3U 文件内容
   * @param config 配置解析的字段映射，默认为 DEFAULT_CONFIG
   * @returns 按分组整理的频道列表，格式为 [{ groupName: '分组名', channels: [...] }]
   */
  static parse(
    m3uContent: string,
    config: ParseConfig = DEFAULT_CONFIG
  ): M3UGroup[] {
    const lines = m3uContent.split("\n").map((line) => line.trim());
    const groups: Record<string, M3UChannel[]> = {};
    let currentAttributes: Record<string, string> = {};

    for (const line of lines) {
      if (line.startsWith("#EXTINF")) {
        // 提取属性字段
        currentAttributes = this.extractAttributes(line, config);
      } else if (line && !line.startsWith("#")) {
        // 当前行是 URL
        const url = line;
        const name = this.getFirstMatch(currentAttributes, config.nameFields);
        const logo = this.getFirstMatch(currentAttributes, config.logoFields);
        const group = this.getFirstMatch(
          currentAttributes,
          config.groupFields,
          "默认分组"
        );

        const channel: M3UChannel = { url, name, logo };

        // 将频道添加到分组中
        if (!groups[group]) {
          groups[group] = [];
        }
        groups[group].push(channel);

        // 重置属性
        currentAttributes = {};
      }
    }

    // 转换为目标格式
    return Object.entries(groups).map(([groupName, channels]) => ({
      groupName,
      channels,
    }));
  }

  static flattenChannels(data: any[], sourceId: number): any[] {
    const result: any[] = [];

    data.forEach((group) => {
      const groupName = group.groupName;
      group.channels.forEach((channel: any) => {
        result.push({
          ...channel,
          group_name: groupName,
          source_id: sourceId,
        });
      });
    });

    return result;
  }

  static formatChannels(channels: any[]): any[] {
    const groups: Record<string, M3UChannel[]> = {};
    channels.forEach((channel) => {
      channel.groupName = channel.group_name;
      delete channel.group_name;
      const group = channel.groupName;
      // 将频道添加到分组中
      if (!groups[group]) {
        groups[group] = [];
      }
      groups[group].push(channel);
    });

    // 转换为目标格式
    return Object.entries(groups).map(([groupName, channels]) => ({
      groupName,
      channels,
    }));
  }

  /**
   * 提取 #EXTINF 属性字段
   * @param line #EXTINF 行
   * @param config 配置解析的字段映射
   * @returns 属性字段的键值对
   */
  private static extractAttributes(
    line: string,
    config: ParseConfig
  ): Record<string, string> {
    const attributes: Record<string, string> = {};
    const regex = /(\w[\w-]*)="([^"]*)"/g;
    let match;
    while ((match = regex.exec(line)) !== null) {
      attributes[match[1]] = match[2];
    }
    return attributes;
  }

  /**
   * 根据字段名列表依次匹配返回第一个值
   * @param attributes 属性字段
   * @param fields 字段名列表
   * @param defaultValue 如果未匹配到，返回的默认值
   * @returns 匹配到的值或默认值
   */
  private static getFirstMatch(
    attributes: Record<string, string>,
    fields: string[],
    defaultValue: string = ""
  ): string {
    for (const field of fields) {
      if (attributes[field]) {
        return attributes[field];
      }
    }
    return defaultValue;
  }
}
