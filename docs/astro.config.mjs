// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
	site: 'https://ntic-solution.github.io',
	base: '/syngard',
	telemetry: false,
	integrations: [
		starlight({
			title: 'Syngard',
			description: 'Local-first integrity control for AI agent instructions.',
			favicon: '/favicon.svg',
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/NTIC-Solution/syngard' },
			],
			editLink: {
				baseUrl: 'https://github.com/NTIC-Solution/syngard/edit/main/docs/',
			},
			defaultLocale: 'root',
			locales: {
				root: { label: 'English', lang: 'en' },
				fr: { label: 'Français', lang: 'fr' },
			},
			sidebar: [
				{
					label: 'Getting started',
					translations: { fr: 'Prise en main' },
					items: [
						{ slug: 'getting-started/installation' },
						{ slug: 'getting-started/quick-start' },
					],
				},
				{
					label: 'Concepts',
					items: [
						{ slug: 'concepts/how-it-works' },
						{ slug: 'concepts/security-model' },
					],
				},
				{
					label: 'Guides',
					items: [{ slug: 'guides/ci' }],
				},
				{
					label: 'Reference',
					translations: { fr: 'Référence' },
					items: [
						{ slug: 'reference/cli' },
						{ slug: 'reference/manifest' },
						{ slug: 'reference/json-reports' },
					],
				},
				{
					label: 'Project',
					translations: { fr: 'Projet' },
					items: [{ slug: 'project/roadmap' }],
				},
			],
		}),
	],
});
