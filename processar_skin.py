"""
Importador/Processador de Skins da Aster Companion (Experimental).
Usa rembg (modelo isnet-anime) para remover fundo branco/opaco de sprites 2D anime,
aplica contorno preto suave opcional, alinha em 720px e extrai ciclo de walk de MP4.
"""
import argparse
import os
import sys
from collections import deque
import numpy as np
from PIL import Image, ImageEnhance, ImageFilter

NAME_MAP = {
    "idle": "aster_idle.png",
    "aster_idle": "aster_idle.png",
    "blink": "aster_blink.png",
    "aster_blink": "aster_blink.png",
    "talk": "aster_talk.png",
    "aster_talk": "aster_talk.png",
    "talk_calm": "aster_talk_calm.png",
    "aster_talk_calm": "aster_talk_calm.png",
    "calm_talk": "aster_talk_calm.png",
    "smug": "aster_smug.png",
    "aster_smug": "aster_smug.png",
    "angry": "aster_angry.png",
    "aster_angry": "aster_angry.png",
    "think": "aster_think.png",
    "aster_think": "aster_think.png",
    "sleep": "aster_sleepy.png",
    "aster_sleep": "aster_sleepy.png",
    "sleepy": "aster_sleepy.png",
    "aster_sleepy": "aster_sleepy.png",
    "listen": "aster_listen.png",
    "aster_listen": "aster_listen.png",
    "music": "aster_music.png",
    "aster_music": "aster_music.png",
    "drag": "aster_drag.png",
    "aster_drag": "aster_drag.png",
    "sit": "aster_sit.png",
    "aster_sit": "aster_sit.png",
}

_REMBG_SESSION = None


def get_anime_session():
    global _REMBG_SESSION
    if _REMBG_SESSION is None:
        from rembg import new_session
        _REMBG_SESSION = new_session("isnet-anime")
    return _REMBG_SESSION


def has_transparent_corners(im: Image.Image) -> bool:
    rgba = im.convert("RGBA")
    arr = np.array(rgba)
    h, w, _ = arr.shape
    corners = [arr[2, 2, 3], arr[2, w - 3, 3], arr[h - 3, 2, 3], arr[h - 3, w - 3, 3]]
    return all(int(a) < 20 for a in corners)


def remove_bg_anime(im: Image.Image) -> Image.Image:
    from rembg import remove
    rgba = im.convert("RGBA")
    if has_transparent_corners(rgba):
        return rgba
    session = get_anime_session()
    out = remove(rgba, session=session, post_process_mask=True)
    arr = np.array(out.convert("RGBA"), dtype=np.uint8)
    alpha = arr[:, :, 3]
    # Limpa resquícios quase transparentes
    arr[alpha < 35, 3] = 0
    # Defringe: escurece pixels semitransparentes claros na borda para evitar halo cinza/branco dentro do contorno
    semi = (alpha >= 35) & (alpha < 175)
    if np.any(semi):
        r = arr[:, :, 0].astype(np.int16)
        g = arr[:, :, 1].astype(np.int16)
        b = arr[:, :, 2].astype(np.int16)
        cmin = np.minimum(np.minimum(r, g), b)
        bright_semi = semi & (cmin >= 150)
        if np.any(bright_semi):
            arr[bright_semi, 0] = (arr[bright_semi, 0].astype(np.float32) * 0.25).astype(np.uint8)
            arr[bright_semi, 1] = (arr[bright_semi, 1].astype(np.float32) * 0.25).astype(np.uint8)
            arr[bright_semi, 2] = (arr[bright_semi, 2].astype(np.float32) * 0.25).astype(np.uint8)
    return Image.fromarray(arr, "RGBA")


def add_black_outline(im: Image.Image, thickness: int = 2) -> Image.Image:
    if thickness <= 0:
        return im
    rgba = im.convert("RGBA")
    alpha = rgba.split()[3]
    ksize = max(3, thickness * 2 + 1)
    dilated = alpha.filter(ImageFilter.MaxFilter(ksize))
    dilated = dilated.filter(ImageFilter.GaussianBlur(radius=0.55))

    outline_layer = Image.new("RGBA", rgba.size, (14, 12, 18, 0))
    outline_layer.putalpha(dilated)
    combined = Image.alpha_composite(outline_layer, rgba)
    return combined


def remove_chroma_flood(im: Image.Image) -> Image.Image:
    rgba = im.convert("RGBA")
    arr = np.array(rgba, dtype=np.uint8)
    h, w, _ = arr.shape
    r = arr[:, :, 0].astype(np.float32)
    g = arr[:, :, 1].astype(np.float32)
    b = arr[:, :, 2].astype(np.float32)

    corners = np.array([
        arr[4, 4, :3], arr[4, w - 5, :3],
        arr[h - 5, 4, :3], arr[h - 5, w - 5, :3]
    ], dtype=np.float32)
    bg_rgb = np.median(corners, axis=0)

    is_chroma_blue = bg_rgb[2] > bg_rgb[0] + 60 and bg_rgb[2] > 140 and bg_rgb[2] > bg_rgb[1]
    is_chroma_green = bg_rgb[1] > bg_rgb[0] + 60 and bg_rgb[1] > bg_rgb[2] + 50 and bg_rgb[1] > 140
    if not is_chroma_blue and not is_chroma_green:
        return remove_bg_anime(im)

    dist = np.sqrt((r - bg_rgb[0]) ** 2 + (g - bg_rgb[1]) ** 2 + (b - bg_rgb[2]) ** 2)
    is_bg = dist < 80.0

    visited = np.zeros((h, w), dtype=bool)
    bg_mask = np.zeros((h, w), dtype=bool)
    q = deque()
    for x in range(w):
        for y in (0, h - 1):
            if is_bg[y, x] and not visited[y, x]:
                visited[y, x] = True
                q.append((y, x))
    for y in range(h):
        for x in (0, w - 1):
            if is_bg[y, x] and not visited[y, x]:
                visited[y, x] = True
                q.append((y, x))
    while q:
        y, x = q.popleft()
        bg_mask[y, x] = True
        for ny, nx in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)):
            if 0 <= ny < h and 0 <= nx < w and not visited[ny, nx] and is_bg[ny, nx]:
                visited[ny, nx] = True
                q.append((ny, nx))

    bg_mask |= (dist < 62.0) & (~bg_mask)

    out = arr.copy()
    out[:, :, 3] = 255
    out[bg_mask, 3] = 0

    pad = np.pad(bg_mask, 1, mode="constant", constant_values=True)
    edge1 = (pad[:-2, 1:-1] | pad[2:, 1:-1] | pad[1:-1, :-2] | pad[1:-1, 2:]) & (~bg_mask)
    if is_chroma_blue:
        chroma_edge = edge1 & (b > np.maximum(r, g) + 25) & (dist < 130.0)
        out[chroma_edge, 3] = np.clip((dist[chroma_edge] - 72.0) / 58.0 * 255.0, 0, 255).astype(np.uint8)
        out[chroma_edge, 2] = np.minimum(out[chroma_edge, 2], np.maximum(out[chroma_edge, 0], out[chroma_edge, 1]))
    else:
        chroma_edge = edge1 & (g > np.maximum(r, b) + 25) & (dist < 135.0)
        out[chroma_edge, 3] = np.clip((dist[chroma_edge] - 72.0) / 62.0 * 255.0, 0, 255).astype(np.uint8)
        out[chroma_edge, 1] = np.minimum(out[chroma_edge, 1], np.maximum(out[chroma_edge, 0], out[chroma_edge, 2]))

    return Image.fromarray(out, "RGBA")


def process_walk_mp4(mp4_path: str, out_dir: str, outline: int = 0, target_h: int = 720) -> int:
    import av
    c = av.open(mp4_path)
    frames = [f.to_image() for f in c.decode(video=0)]
    c.close()
    n = len(frames)
    if n < 8:
        return 0

    small = [np.array(f.resize((90, 160)), dtype=np.float32) for f in frames]
    best = (1e18, min(12, n // 4), min(24, max(8, n // 3)))
    for s in range(10, min(42, n - 18), 2):
        for p in range(20, min(52, n - s - 2)):
            d = (
                np.mean(np.abs(small[s] - small[s + p])) +
                np.mean(np.abs(small[s + 1] - small[s + p + 1])) +
                np.mean(np.abs(small[s + 2] - small[s + p + 2]))
            ) / 3.0
            if d < best[0]:
                best = (float(d), s, p)

    _, s0, p0 = best
    indices = [s0 + int(round(i * p0 / 8.0)) for i in range(8)]
    cleaned = []
    for idx in indices:
        im = remove_chroma_flood(frames[min(idx, n - 1)])
        # Compensa leve perda de contraste/saturação típica de compressão MP4
        r, g, b, a = im.split()
        rgb = Image.merge("RGB", (r, g, b))
        rgb = ImageEnhance.Contrast(rgb).enhance(1.08)
        rgb = ImageEnhance.Color(rgb).enhance(1.12)
        rgb = ImageEnhance.Sharpness(rgb).enhance(1.10)
        r2, g2, b2 = rgb.split()
        im = Image.merge("RGBA", (r2, g2, b2, a))
        if outline > 0:
            im = add_black_outline(im, outline)
        cleaned.append(im)

    ux0, uy0 = 10**9, 10**9
    ux1, uy1 = 0, 0
    for im in cleaned:
        arr = np.array(im)
        ys, xs = np.where(arr[:, :, 3] > 20)
        if len(ys) == 0:
            continue
        ux0 = min(ux0, int(xs.min()))
        uy0 = min(uy0, int(ys.min()))
        ux1 = max(ux1, int(xs.max()) + 1)
        uy1 = max(uy1, int(ys.max()) + 1)

    if ux1 <= ux0 or uy1 <= uy0:
        return 0

    uw = ux1 - ux0
    uh = uy1 - uy0
    scale = target_h / float(uh)
    nw = int(round(uw * scale))
    canvas_w = max(390, nw)
    px = (canvas_w - nw) // 2

    for i, im in enumerate(cleaned, start=1):
        cropped = im.crop((ux0, uy0, ux1, uy1))
        resized = cropped.resize((nw, target_h), Image.Resampling.LANCZOS)
        canvas = Image.new("RGBA", (canvas_w, target_h), (0, 0, 0, 0))
        canvas.paste(resized, (px, 0), resized)
        canvas.save(os.path.join(out_dir, f"aster_walk_{i}.png"), "PNG", optimize=True)
    return 8


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input-dir", required=True)
    parser.add_argument("--skin-dir", required=True)
    parser.add_argument("--outline", type=int, default=2)
    args = parser.parse_args()

    os.makedirs(args.skin_dir, exist_ok=True)
    imported_static = 0
    imported_walk = 0
    mp4_file = None

    for fname in sorted(os.listdir(args.input_dir)):
        fpath = os.path.join(args.input_dir, fname)
        if not os.path.isfile(fpath):
            continue
        stem, ext = os.path.splitext(fname)
        ext_l = ext.lower()
        stem_l = stem.lower().strip()

        if ext_l == ".mp4" and mp4_file is None:
            mp4_file = fpath
            continue

        if ext_l not in (".png", ".webp", ".jpg", ".jpeg"):
            continue

        target_name = NAME_MAP.get(stem_l)
        if not target_name:
            continue

        im = Image.open(fpath)
        was_transparent = has_transparent_corners(im)
        cleaned = remove_bg_anime(im)
        if not was_transparent and args.outline > 0:
            cleaned = add_black_outline(cleaned, args.outline)

        w, h = cleaned.size
        target_h = 720
        target_w = int(round(w * (target_h / float(h))))
        resized = cleaned.resize((target_w, target_h), Image.Resampling.LANCZOS)
        resized.save(os.path.join(args.skin_dir, target_name), "PNG", optimize=True)
        imported_static += 1

    if mp4_file:
        imported_walk = process_walk_mp4(mp4_file, args.skin_dir, outline=0, target_h=720)

    print(f"OK: {imported_static} sprites estáticos e {imported_walk} frames de caminhada importados.")


if __name__ == "__main__":
    main()
