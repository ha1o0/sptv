import CryptoJS from 'crypto-js';

// 将 URL 转换为 MD5 哈希
export const hashUrl = (url: string) => {
  return CryptoJS.MD5(url).toString();
};