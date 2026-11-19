// Reads a QR code out of an image, for importing servers from a screenshot or a provider's
// picture. Decoding runs here in the webview, so the app needs no image libraries in Rust.

import jsQR from 'jsqr';

/** First pass size: fast, and enough for a code that fills a good part of the picture. */
const FIRST_PASS = 1600;
/** Second pass size, for a small code inside a large screenshot. */
const FULL_PASS = 4000;

const IMAGE_EXTENSIONS = /\.(png|jpe?g|gif|webp|bmp|heic|heif|tiff?)$/i;

/** The size that fits `max` on its longer side, never larger than the image itself. */
export function fitWithin(width: number, height: number, max: number): { width: number; height: number } {
	const scale = Math.min(1, max / Math.max(width, height));
	return { width: Math.max(1, Math.round(width * scale)), height: Math.max(1, Math.round(height * scale)) };
}

/** The text of the first QR code in RGBA pixels, or `null`. Light-on-dark codes are read too. */
export function decodeRgba(data: Uint8ClampedArray, width: number, height: number): string | null {
	const code = jsQR(data, width, height, { inversionAttempts: 'attemptBoth' });
	const text = code?.data.trim();
	return text ? text : null;
}

export function isImageFile(file: { name: string; type: string }): boolean {
	return file.type.startsWith('image/') || IMAGE_EXTENSIONS.test(file.name);
}

/** Decodes an image file or blob; `null` when it holds no readable code. */
export async function decodeImage(blob: Blob): Promise<string | null> {
	const bitmap = await createImageBitmap(blob);
	try {
		const first = decodeBitmap(bitmap, FIRST_PASS);
		if (first || Math.max(bitmap.width, bitmap.height) <= FIRST_PASS) return first;
		return decodeBitmap(bitmap, FULL_PASS);
	} finally {
		bitmap.close();
	}
}

function decodeBitmap(bitmap: ImageBitmap, max: number): string | null {
	const { width, height } = fitWithin(bitmap.width, bitmap.height, max);
	const canvas = document.createElement('canvas');
	canvas.width = width;
	canvas.height = height;
	const context = canvas.getContext('2d', { willReadFrequently: true });
	if (!context) return null;
	// A transparent background would read as black and hide a dark code.
	context.fillStyle = '#fff';
	context.fillRect(0, 0, width, height);
	context.drawImage(bitmap, 0, 0, width, height);
	return decodeRgba(context.getImageData(0, 0, width, height).data, width, height);
}

/** A single web address with nothing else, which in a QR code is usually a subscription. */
export function webAddress(text: string): string | null {
	const trimmed = text.trim();
	if (!/^https?:\/\/\S+$/i.test(trimmed)) return null;
	try {
		new URL(trimmed);
		return trimmed;
	} catch {
		return null;
	}
}
