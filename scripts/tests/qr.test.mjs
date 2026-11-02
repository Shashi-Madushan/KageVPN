import { test } from 'node:test';
import assert from 'node:assert/strict';
import qrcode from 'qrcode-generator';
import { decodeRgba, fitWithin, isImageFile, webAddress } from '../../src/lib/qr.ts';

/** RGBA pixels of a QR code for `text`, `cell` pixels per module with a quiet zone. */
function render(text, { cell = 4, invert = false } = {}) {
	const qr = qrcode(0, 'M');
	qr.addData(text);
	qr.make();
	const modules = qr.getModuleCount();
	const size = (modules + 8) * cell;
	const data = new Uint8ClampedArray(size * size * 4);
	for (let y = 0; y < size; y++) {
		for (let x = 0; x < size; x++) {
			const mx = Math.floor(x / cell) - 4;
			const my = Math.floor(y / cell) - 4;
			const dark = mx >= 0 && my >= 0 && mx < modules && my < modules && qr.isDark(my, mx);
			const v = dark !== invert ? 0 : 255;
			data.set([v, v, v, 255], (y * size + x) * 4);
		}
	}
	return { data, width: size, height: size };
}

const link = 'vless://11111111-2222-3333-4444-555555555555@203.0.113.7:443?security=tls&type=ws#Test';

test('a QR code with a server link is read back', () => {
	const { data, width, height } = render(link);
	assert.equal(decodeRgba(data, width, height), link);
});

test('a light-on-dark QR code is read too', () => {
	const { data, width, height } = render(link, { invert: true });
	assert.equal(decodeRgba(data, width, height), link);
});

test('an image without a code gives nothing', () => {
	const blank = new Uint8ClampedArray(200 * 200 * 4).fill(255);
	assert.equal(decodeRgba(blank, 200, 200), null);
});

test('large images are scaled to fit and small ones are left alone', () => {
	assert.deepEqual(fitWithin(4000, 2000, 1600), { width: 1600, height: 800 });
	assert.deepEqual(fitWithin(900, 1800, 1600), { width: 800, height: 1600 });
	assert.deepEqual(fitWithin(300, 200, 1600), { width: 300, height: 200 });
});

test('image files are told apart from text files', () => {
	assert.ok(isImageFile({ name: 'code.png', type: 'image/png' }));
	assert.ok(isImageFile({ name: 'Screenshot.HEIC', type: '' }));
	assert.ok(!isImageFile({ name: 'servers.txt', type: 'text/plain' }));
	assert.ok(!isImageFile({ name: 'wg0.conf', type: '' }));
});

test('a lone web address is recognised as a likely subscription', () => {
	assert.equal(webAddress(' https://provider.example/sub?token=abc \n'), 'https://provider.example/sub?token=abc');
	assert.equal(webAddress(link), null);
	assert.equal(webAddress('https://a.example/x https://b.example/y'), null);
	assert.equal(webAddress('ftp://a.example/x'), null);
});
