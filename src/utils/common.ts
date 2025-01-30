/**
 * 判断字符串是否为时间格式
 * @param {string} value
 * @returns {boolean}
 */
export function isDateTime(value: string) {
    // 匹配常见的时间格式
    const dateTimeRegex = /^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(Z)?$/;
    return dateTimeRegex.test(value);
}

/**
 * 将时间字符串转换为本地时间
 * @param {string} value
 * @returns {string}
 */
export function toLocalTime(value: string) {
    if (isDateTime(value)) {
        // 如果时间格式包含空格，替换为 "T" 以符合 Date 解析要求
        const formattedValue = value.includes(" ") ? value.replace(" ", "T") + "Z" : value;
        return new Date(formattedValue).toLocaleString();
    }
    return value; // 如果不是时间格式，返回原值
}

/**
 * 递归转换对象或数组中的时间字段
 * @param {object|array} data
 * @returns {object|array}
 */
export function convertTimestampsDeep(data: any[] | null) {
    if (Array.isArray(data)) {
        return data.map(item => convertTimestampsDeep(item));
    } else if (typeof data === "object" && data !== null) {
        const result = {};
        for (const key in data) {
            if (typeof data[key] === "string") {
                result[key] = toLocalTime(data[key]);
            } else if (typeof data[key] === "object" || Array.isArray(data[key])) {
                result[key] = convertTimestampsDeep(data[key]); // 递归处理嵌套对象或数组
            } else {
                result[key] = data[key];
            }
        }
        return result;
    }
    return data;
}