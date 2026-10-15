import type { RouteAction, RuleSetKind } from './api';

/** A list people can pick by name instead of knowing its file name. */
export interface CatalogEntry {
	kind: RuleSetKind;
	/** The country code or category as the list sources name it. */
	name: string;
	title: string;
	/** What the list covers, in a few words. */
	hint: string;
	/** What most people want done with it. */
	suggested: RouteAction;
}

const country = (name: string, title: string): CatalogEntry => ({ kind: 'geoip', name, title, hint: `Addresses located in ${title}`, suggested: 'direct' });
const site = (name: string, title: string, hint: string, suggested: RouteAction = 'proxy'): CatalogEntry => ({ kind: 'geosite', name, title, hint, suggested });

/** Countries, by the network addresses registered there. */
export const COUNTRIES: CatalogEntry[] = [
	country('cn', 'China'),
	country('ru', 'Russia'),
	country('ir', 'Iran'),
	country('lk', 'Sri Lanka'),
	country('in', 'India'),
	country('pk', 'Pakistan'),
	country('bd', 'Bangladesh'),
	country('tr', 'Turkey'),
	country('sa', 'Saudi Arabia'),
	country('ae', 'United Arab Emirates'),
	country('eg', 'Egypt'),
	country('ua', 'Ukraine'),
	country('kz', 'Kazakhstan'),
	country('vn', 'Vietnam'),
	country('id', 'Indonesia'),
	country('hk', 'Hong Kong'),
	country('jp', 'Japan'),
	country('kr', 'South Korea'),
	country('us', 'United States')
];

/** Well-known websites and services, grouped by what they are. */
export const CATEGORIES: CatalogEntry[] = [
	site('category-ads-all', 'Advertising and trackers', 'Ad networks and trackers', 'block'),
	site('private', 'Local and private networks', 'Home, office and other private addresses', 'direct'),
	site('cn', 'Chinese websites', 'Sites that are mainly used in China', 'direct'),
	site('geolocation-!cn', 'Websites outside China', 'Sites that are not Chinese', 'proxy'),
	site('google', 'Google', 'Search, Gmail, Maps and more'),
	site('youtube', 'YouTube', 'Video'),
	site('apple', 'Apple', 'iCloud, App Store and more'),
	site('microsoft', 'Microsoft', 'Windows, Office and Azure'),
	site('github', 'GitHub', 'Code hosting'),
	site('openai', 'OpenAI and ChatGPT', 'AI assistant'),
	site('anthropic', 'Anthropic and Claude', 'AI assistant'),
	site('netflix', 'Netflix', 'Streaming'),
	site('disney', 'Disney+', 'Streaming'),
	site('spotify', 'Spotify', 'Music'),
	site('telegram', 'Telegram', 'Messaging'),
	site('whatsapp', 'WhatsApp', 'Messaging'),
	site('discord', 'Discord', 'Chat and communities'),
	site('twitter', 'X (Twitter)', 'Social network'),
	site('facebook', 'Facebook and Instagram', 'Social network'),
	site('tiktok', 'TikTok', 'Short video'),
	site('steam', 'Steam', 'Games store'),
	site('category-games', 'Games', 'Game services'),
	site('paypal', 'PayPal', 'Payments'),
	site('amazon', 'Amazon', 'Shopping and cloud')
];

export const CATALOG: CatalogEntry[] = [...COUNTRIES, ...CATEGORIES];

export function findEntry(kind: RuleSetKind, name: string): CatalogEntry | undefined {
	const wanted = name.trim().toLowerCase();
	return CATALOG.find((e) => e.kind === kind && e.name === wanted);
}

/** How a list is shown: a name people recognise and one line on what it covers. */
export function describeList(kind: RuleSetKind, name: string): { title: string; hint: string } {
	const entry = findEntry(kind, name);
	if (entry) return { title: entry.title, hint: kind === 'geoip' ? entry.hint : entry.hint };
	return { title: name.toUpperCase(), hint: kind === 'geoip' ? 'Custom list of network addresses' : 'Custom list of websites' };
}
