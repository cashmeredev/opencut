import type { NextConfig } from "next";
import { withContentCollections } from "@content-collections/next";

const nextConfig: NextConfig = {
	compiler: {
		removeConsole: process.env.NODE_ENV === "production",
	},
	devIndicators: false,
	reactStrictMode: true,
	productionBrowserSourceMaps: true,
	output: "export",
	trailingSlash: true,
	images: {
		unoptimized: true,
	},
};

export default withContentCollections(nextConfig);
