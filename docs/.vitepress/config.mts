import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
	lang: "en-US",
	title: "PicoLimbo",
	description:
		"An ultra-lightweight, multi-version Minecraft limbo server written in Rust",
	sitemap: {
		hostname: "https://picolimbo.quozul.dev",
	},
	head: [["link", { rel: "icon", href: "/favicon.png" }]],
	themeConfig: {
		// https://vitepress.dev/reference/default-theme-config
		nav: [
			{ text: "Home", link: "/" },
			{ text: "Docs", link: "/about/introduction.html" },
		],
		sidebar: [
			{
				text: "About",
				items: [
					{ text: "Introduction", link: "/about/introduction.html" },
					{
						text: "Supported Versions",
						link: "/about/supported-versions.html",
					},
					{ text: "Installation", link: "/about/installation.html" },
					{ text: "CLI Usage", link: "/about/cli-usage.html" },
					{ text: "FAQ", link: "/about/faq.html" },
					{ text: "Benchmarks", link: "/about/benchmarks.html" },
				],
			},
			{
				text: "Configuration",
				items: [
					{ text: "Introduction", link: "/config/introduction.html" },
					{ text: "Server Settings", link: "/config/server-settings.html" },
                    { text: "World Configuration", link: "/config/world.html" },
					{ text: "Proxy Integration", link: "/config/proxy-integration.html" },
					{ text: "Server List", link: "/config/server-list.html" },
					{ text: "Tab List", link: "/config/tab-list.html" },
                    { text: "Boss Bar", link: "/config/boss-bar.html" },
					{ text: "Experimental World Features", link: "/config/experimental-world.html" },
					{
						text: "Default Configuration",
						link: "/config/default-configuration.html",
					},
				],
			},
			{
				text: "Customization",
				items: [
					{ text: "Message Formatting", link: "/customization/message-formatting.html" },
				],
			},
		],
		socialLinks: [
			{ icon: "github", link: "https://github.com/Quozul/PicoLimbo" },
			{ icon: "discord", link: "https://discord.gg/M2a9dxJPRy" },
		],
		search: {
			provider: "local",
		},
		footer: {
			message: "Released under the MIT License.",
		},
	},
});
