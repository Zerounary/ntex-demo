export const assetsUrl = (path = '') => {
  return `${import.meta.env.VITE_ASSETS_URL}/${path}`;
};

export const storage =
  typeof global === 'undefined' ? localStorage : ({} as Storage);

export const HanMap = n => {
  let han = '零一二三四五六七八九十'.split('');
  let len = han.length
  for (let i = 1; i < len - 1; i++) {
    han.push('十' + han[i]);
  }
  return han[n];
};
