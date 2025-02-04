const proxyPrefix = "http://localhost:3000/proxy?url=";
const configProxy = (url: string) => {
    if (!url.startsWith('http')) {
        return url;
    }
    return proxyPrefix + encodeURIComponent(url);
}

export { proxyPrefix, configProxy };