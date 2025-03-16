// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Marvin Light Control',
			logo: {
				src: "./src/assets/icon.png",
			},
			social: {
				github: 'https://github.com/PixelboysTM/marvin_light_control',
			},
			customCss: ["./src/styles/custom.css",],
			sidebar: [
				{
					label: 'Guides',
					autogenerate: { directory: 'guides' },
				},
				{
					label: 'Reference',
					autogenerate: { directory: 'reference' },
				},
			],
		}),
	],
});
