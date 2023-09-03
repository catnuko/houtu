import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "houtu",
  description: "webgpu based high performance 3D earth rendering engine",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'ChangeLog', link: '/changelog' }
    ],

    sidebar: [
      // {
      //   text: 'News',
      //   items: [
      //     { text: 'Markdown Examples', link: '/markdown-examples' },
      //   ]
      // }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/catnuko/houtu' }
    ],
  },
  locales: {
    root: {
      label: 'English',
      lang: 'en'
    },
    zh: {
      label: '简体中文',
      lang: 'zh', // optional, will be added  as `lang` attribute on `html` tag
    },
  }
})
