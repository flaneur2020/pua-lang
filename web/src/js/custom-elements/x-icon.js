const icons = {
  play: (color) => `
<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg">
  <path d="M6.4 11.6L11.2 8 6.4 4.4v7.2zM8 0C3.584 0 0 3.584 0 8s3.584 8 8 8 8-3.584 8-8-3.584-8-8-8zm0 14.4A6.408 6.408 0 0 1 1.6 8c0-3.528 2.872-6.4 6.4-6.4 3.528 0 6.4 2.872 6.4 6.4 0 3.528-2.872 6.4-6.4 6.4z" fill="${color}" fill-rule="nonzero"/>
</svg>
`,
  sort: (color) => `
<svg viewBox="0 0 12 8" xmlns="http://www.w3.org/2000/svg">
  <g fill="none" fill-rule="evenodd">
    <path d="M-2-4h16v16H-2z"/>
    <path d="M0 8h4V6.667H0V8zm0-8v1.333h12V0H0zm0 4.667h8V3.333H0v1.334z" fill="${color}" fill-rule="nonzero"/>
  </g>
</svg>
`,
  share: (color) => `
<svg viewBox="0 0 12 13" xmlns="http://www.w3.org/2000/svg">
  <g fill="none" fill-rule="evenodd">
    <path d="M-2-1h16v16H-2z"/>
    <path d="M10 9.189c-.507 0-.96.196-1.307.502L3.94 6.983c.033-.15.06-.3.06-.457 0-.157-.027-.307-.06-.457l4.7-2.682c.36.326.833.529 1.36.529 1.107 0 2-.875 2-1.958C12 .874 11.107 0 10 0S8 .874 8 1.958c0 .156.027.307.06.457l-4.7 2.682A2.015 2.015 0 0 0 2 4.568c-1.107 0-2 .875-2 1.958s.893 1.958 2 1.958c.527 0 1-.202 1.36-.529l4.747 2.715c-.034.137-.054.28-.054.424C8.053 12.145 8.927 13 10 13s1.947-.855 1.947-1.906c0-1.05-.874-1.905-1.947-1.905z" fill="${color}" fill-rule="nonzero"/>
  </g>
</svg>
`,
  doc: (color) => `
<svg viewBox="0 0 12 14" xmlns="http://www.w3.org/2000/svg">
  <g fill="none" fill-rule="evenodd">
    <path d="M-2-1h16v16H-2z"/>
    <path d="M7.333.333H2c-.733 0-1.327.6-1.327 1.334L.667 12.333c0 .734.593 1.334 1.326 1.334H10c.733 0 1.333-.6 1.333-1.334v-8l-4-4zM8.667 11H3.333V9.667h5.334V11zm0-2.667H3.333V7h5.334v1.333zM6.667 5V1.333L10.333 5H6.667z" fill="${color}" fill-rule="nonzero"/>
  </g>
</svg>
`,
  comment: (color) => `
<svg viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
  <g fill="none" fill-rule="evenodd">
    <path d="M-2-2h24v24H-2z"/>
    <path d="M18 0H2C.9 0 .01.9.01 2L0 20l4-4h14c1.1 0 2-.9 2-2V2c0-1.1-.9-2-2-2zM7 9H5V7h2v2zm4 0H9V7h2v2zm4 0h-2V7h2v2z" fill="${color}" fill-rule="nonzero"/>
  </g>
</svg>
`,
  github: (color) => `
<svg viewBox="0 0 33 32" xmlns="http://www.w3.org/2000/svg">
  <path d="M16.288 0C7.294 0 0 7.293 0 16.29c0 7.197 4.667 13.302 11.14 15.456.815.15 1.112-.353 1.112-.785 0-.387-.014-1.411-.022-2.77-4.531.984-5.487-2.184-5.487-2.184-.741-1.882-1.809-2.383-1.809-2.383-1.479-1.01.112-.99.112-.99 1.635.115 2.495 1.679 2.495 1.679 1.453 2.489 3.813 1.77 4.741 1.353.148-1.052.569-1.77 1.034-2.177-3.617-.411-7.42-1.809-7.42-8.051 0-1.778.635-3.233 1.677-4.371-.168-.412-.727-2.069.16-4.311 0 0 1.367-.438 4.479 1.67a15.602 15.602 0 0 1 4.078-.549 15.62 15.62 0 0 1 4.078.549c3.11-2.108 4.475-1.67 4.475-1.67.889 2.242.33 3.899.163 4.311 1.044 1.138 1.674 2.593 1.674 4.371 0 6.258-3.809 7.635-7.437 8.038.584.503 1.105 1.497 1.105 3.016 0 2.178-.02 3.935-.02 4.469 0 .436.294.943 1.12.784 6.468-2.159 11.131-8.26 11.131-15.455C32.579 7.293 25.285 0 16.288 0" fill="${color}" fill-rule="evenodd"/>
</svg>
`,
};

const defaultIconFunc = () => '';

export class Icon extends HTMLElement {
  static get observedAttributes() {
    return ['icon', 'size', 'color'];
  }

  connectedCallback() {
    this.root = this.attachShadow({ mode: 'open' });
    this.render();
  }

  attributeChangedCallback(attr, previousValue, nextValue) {
    if (this.root) {
      this.render();
    }
  }

  render() {
    const icon = icons[this.attributes.icon.value] || defaultIconFunc;

    this.root.innerHTML = `
<style>
:host {
  display: inline-block;
  width: ${this.attributes.size.value}px;
  height: ${this.attributes.size.value}px;
  vertical-align: middle;
  color: ${this.attributes.color.value};
}

svg {
  width: 100%;
  height: 100%;
  vertical-align: top;
}
</style>

${icon(this.attributes.color.value)}
`;
  }
}
