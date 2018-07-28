import LZString from 'lz-string';
import { SHARE_QUERY_KEY } from '../constants';
import { Command } from './';

const share = document.getElementById('share');
const tooltip = share.querySelector('.tooltip');
let timerId = 0;

share.addEventListener(
  'click',
  (e) => {
    e.preventDefault();

    const { protocol, host, pathname } = window.location;
    const value = LZString.compressToEncodedURIComponent(Command.getValue());

    window.history.pushState({}, '', `${protocol}//${host}${pathname}?${SHARE_QUERY_KEY}=${value}`);

    clearTimeout(timerId);

    tooltip.classList.add('tooltip--active');

    timerId = setTimeout(() => {
      tooltip.classList.remove('tooltip--active');
    }, 4000);
  },
  false,
);
