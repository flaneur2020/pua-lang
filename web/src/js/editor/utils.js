export const zeroPadding = (str, size) => str.padStart(size, '0');

export const date2time = (date) =>
  ['getHours', 'getMinutes', 'getSeconds']
    .map((method) => zeroPadding(`${date[method]()}`, 2))
    .join(':');
