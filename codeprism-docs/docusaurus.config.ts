import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'CodePrism - AI-Powered Code Intelligence MCP Server',
  tagline: 'Production-ready code intelligence for AI assistants. Sponsored by Dragonscale Industries Inc.',
  favicon: 'img/favicon.ico',

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Set the production url of your site here
  url: 'https://rustic-ai.github.io',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/codeprism/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'rustic-ai', // Usually your GitHub org/user name.
  projectName: 'codeprism', // Usually your repo name.

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  themes: ['@docusaurus/theme-mermaid'],
  // In order for Mermaid code blocks to work in Markdown,
  // you also need to enable the Remark plugin with this option
  markdown: {
    mermaid: true,
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/rustic-ai/codeprism/tree/main/codeprism-docs/',
        },
        blog: {
          showReadingTime: true,
          feedOptions: {
            type: ['rss', 'atom'],
            xslt: true,
          },
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/rustic-ai/codeprism/tree/main/codeprism-docs/',
          // Useful options to enforce blogging best practices
          onInlineTags: 'warn',
          onInlineAuthors: 'warn',
          onUntruncatedBlogPosts: 'warn',
        },
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    // Social media meta tags
    image: 'https://cdn.prod.website-files.com/65577aeb720145c27d810263/66296bc4e8282c4a362065f5_logo.svg',
    metadata: [
      // Open Graph meta tags
      {name: 'og:type', content: 'website'},
      {name: 'og:title', content: 'CodePrism - AI-Powered Code Intelligence MCP Server'},
      {name: 'og:description', content: 'Production-ready code intelligence server implementing the Model Context Protocol (MCP) for AI assistants. Graph-based analysis, multi-language support, real-time insights. Sponsored by Dragonscale Industries Inc.'},
      {name: 'og:image', content: 'https://cdn.prod.website-files.com/65577aeb720145c27d810263/66296bc4e8282c4a362065f5_logo.svg'},
      {name: 'og:image:alt', content: 'Dragonscale Industries Inc - Primary Sponsor of CodePrism'},
      {name: 'og:site_name', content: 'CodePrism Documentation'},
      {name: 'og:url', content: 'https://rustic-ai.github.io/codeprism/'},
      
      // Twitter Card meta tags
      {name: 'twitter:card', content: 'summary_large_image'},
      {name: 'twitter:title', content: 'CodePrism - AI-Powered Code Intelligence'},
      {name: 'twitter:description', content: '🤖 100% AI-Generated code intelligence MCP server with graph-based analysis. Sponsored by Dragonscale Industries Inc. 18 production-ready tools for AI assistants.'},
      {name: 'twitter:image', content: 'https://cdn.prod.website-files.com/65577aeb720145c27d810263/66296bc4e8282c4a362065f5_logo.svg'},
      {name: 'twitter:image:alt', content: 'Dragonscale Industries Inc Logo - CodePrism Sponsor'},
      
      // Additional meta tags
      {name: 'description', content: 'Production-ready code intelligence server for AI assistants. Features graph-based code analysis, multi-language support, and real-time insights. 100% AI-generated project sponsored by Dragonscale Industries Inc.'},
      {name: 'keywords', content: 'AI, code intelligence, MCP server, Model Context Protocol, code analysis, graph-based analysis, Dragonscale Industries, AI assistant tools'},
      {name: 'author', content: 'Dragonscale Industries Inc'},
      {name: 'theme-color', content: '#2e8555'},
    ],
    navbar: {
      title: 'CodePrism',
      logo: {
        alt: 'CodePrism Logo',
        src: 'img/logo.png',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {to: '/blog', label: 'Blog', position: 'left'},
        {
          href: 'https://github.com/rustic-ai/codeprism',
          label: 'GitHub',
          position: 'right',
        },
        {
          href: 'https://github.com/sponsors/dragonscale-ai',
          label: '❤️ Sponsor',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Documentation',
          items: [
            {
              label: 'Getting Started',
              to: '/docs/GETTING_STARTED',
            },
            {
              label: 'Architecture',
              to: '/docs/Architecture',
            },
            {
              label: 'API Reference',
              to: '/docs/API_Reference',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub Discussions',
              href: 'https://github.com/rustic-ai/codeprism/discussions',
            },
            {
              label: 'Issues',
              href: 'https://github.com/rustic-ai/codeprism/issues',
            },
            {
              label: 'Contributing',
              to: '/docs/Contributing',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'Blog',
              to: '/blog',
            },
            {
              label: 'GitHub',
              href: 'https://github.com/rustic-ai/codeprism',
            },
            {
              label: '❤️ Sponsor',
              href: 'https://github.com/sponsors/dragonscale-ai',
            },
            {
              label: 'License',
              href: 'https://github.com/rustic-ai/codeprism/blob/main/LICENSE-MIT',
            },
            {
              label: '🏆 Sponsors',
              to: '/docs/Sponsors',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} CodePrism. Sponsored by <a href="https://dragonscale.ai" target="_blank" rel="noopener noreferrer">Dragonscale Industries Inc</a>. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
    mermaid: {
      theme: {light: 'neutral', dark: 'dark'},
      options: {
        // Increase the maximum text length
        maxTextSize: 50000,
        // Set theme variables for better styling
        themeVariables: {
          primaryColor: '#2e8555',
          primaryTextColor: '#1c1e21',
          primaryBorderColor: '#2e8555',
          lineColor: '#606770',
          secondaryColor: '#f1f3f4',
          tertiaryColor: '#ffffff',
        },
      },
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
