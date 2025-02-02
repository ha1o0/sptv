import videojs from "video.js";

export const proxyPrefix = "http://localhost:3000/proxy?url=";

export class CustomVideo {
  static configProxy() {
    videojs.Vhs.xhr.onRequest((options: any) => {
      console.log("options: ", options);

      if (!decodeURIComponent(options.uri).startsWith(proxyPrefix)) {
        options.uri = proxyPrefix + encodeURIComponent(options.uri);
      }

      return options;
    });

    videojs.Vhs.xhr.onResponse((response: any) => {
      console.log("response: ", response);
      let uri = response.uri;
      if (!uri.includes("m3u8")) {
        return response;
      }
      if (uri.startsWith(proxyPrefix)) {
        uri = decodeURIComponent(uri.replace(proxyPrefix, ""));
      }
      const url = new URL(uri);
      const baseUrl = `${url.origin}${url.pathname.substring(
        0,
        url.pathname.lastIndexOf("/") + 1
      )}`;
      console.log("baseUrl: ", baseUrl);
      console.log("url: ", url);
      if (response.responseType !== "text" && response.responseType !== "") {
        return response;
      }

      // 直接读取响应体并修改
      const responseText = response.responseText;
      console.log("body: ", responseText);
      if (responseText && responseText.startsWith("#EXTM3U")) {
        // 获取原始的响应内容
        const originalData = response.responseText;

        // 修改 m3u8 文件内容
        const modifiedData = originalData
          .split("\n")
          .map((line: string) => {
            if (line.includes(".ts") && !line.startsWith("http")) {
              return `${baseUrl}${line}`;
            }
            return line;
          })
          .join("\n");

        console.log("modifiedData: ", modifiedData);
        // 修改 responseText 属性
        Object.defineProperty(response, "responseText", {
          value: modifiedData,
          writable: true,
        });

        Object.defineProperty(response, "response", {
          value: modifiedData,
          writable: true,
        });

        // console.log("modify response: ", response);
      }

      return response;
    });
  }
}
