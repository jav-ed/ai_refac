// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
	site: 'https://refac.javedab.com',
	integrations: [
		starlight({
			title: 'refac',
			description: 'Move source files and keep your project consistent. Language-aware CLI for TypeScript, Python, Go, Rust, Dart, and Markdown.',
			expressiveCode: {
				themes: ['github-dark', 'github-light'],
				styleOverrides: {
					codeFontFamily: "'Maple Mono', monospace",
				},
			},
			components: {
				ThemeProvider: './src/components/ThemeProvider.astro',
			},
			customCss: ['./src/styles/custom.css'],
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/jav-ed/ai_refac' },
			],
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						{ label: 'Overview', slug: 'index' },
						{ label: 'Installation', slug: 'getting-started/installation' },
						{ label: 'Usage', slug: 'getting-started/usage' },
					],
				},
				{
					label: 'Languages',
					items: [
						{ label: 'TypeScript / JavaScript', slug: 'languages/typescript' },
						{ label: 'Python', slug: 'languages/python' },
						{ label: 'Go', slug: 'languages/go' },
						{ label: 'Rust', slug: 'languages/rust' },
						{ label: 'Dart', slug: 'languages/dart' },
						{
							label: 'Markdown',
							items: [
								{ label: 'Overview', slug: 'languages/markdown' },
								{ label: 'Supported Behavior', slug: 'languages/markdown/supported-behavior' },
								{ label: 'Limits & Gaps', slug: 'languages/markdown/limits' },
								{ label: 'Examples', slug: 'languages/markdown/examples' },
							],
						},
					],
				},
				{
					label: 'Reference',
					items: [
						{ label: 'Capabilities & Limits', slug: 'reference/capabilities' },
					],
				},
				{
					label: 'Development',
					items: [
						{ label: 'Build Guide', slug: 'development/build-guide' },
						{ label: 'Testing & Debugging', slug: 'development/testing' },
					],
				},
				{
					label: 'javedab.com ↗',
					link: 'https://javedab.com',
					attrs: { target: '_blank', rel: 'noopener noreferrer' },
				},
			],
		}),
	],
});
