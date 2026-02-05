
#!/usr/bin/env python3
import os
import re
import argparse
from pathlib import Path

IMG_EXTS = {".png", ".jpg", ".jpeg", ".webp"}

# ---- mapeamentos ----
SUIT_EN = {
    "clubs": "c",
    "spades": "s",
    "diamonds": "d",
    "hearts": "h",
}

# Finlandês (pelos seus exemplos)
SUIT_FI = {
    "risti": "c",    # clubs
    "pata": "s",     # spades
    "ruutu": "d",    # diamonds
    "hertta": "h",   # hearts
}

def value_code(n: int) -> str:
    # 1=A, 11=J, 12=Q, 13=K
    if n == 1:
        return "a"
    if n == 11:
        return "j"
    if n == 12:
        return "q"
    if n == 13:
        return "k"
    return str(n)

def parse_target_name(stem: str) -> str | None:
    """
    Recebe o nome do arquivo sem extensão e devolve o novo stem (sem extensão),
    ou None se não reconhecer.
    """
    s = stem.strip()

    low = s.lower()

    # ---- especiais (Pixel_Cards) ----
    if low == "card_blank":
        return "blank"

    if low == "card_back":
        # Pixel_Cards normalmente tem 1 back; escolhi back_b como default.
        # Se quiser outro padrão, mude aqui.
        return "back_b"

    # Joker pixel: Joker_Black / Joker_Red (às vezes sem .png no nome, mas vem no arquivo)
    if low in {"joker_black", "joker black"}:
        # você chamou de joker_b (blue)
        return "joker_b"
    if low in {"joker_red", "joker red"}:
        return "joker_r"

    # ---- especiais (Finlandês) ----
    if low in {"tausta_sininen", "tausta sininen"}:
        return "back_b"
    if low in {"tausta_punainen", "tausta punainen"}:
        return "back_r"
    if low in {"jokeri_musta", "jokeri musta"}:
        return "joker_b"
    if low in {"jokeri_punainen", "jokeri punainen"}:
        return "joker_r"

    # ---- cartas Pixel_Cards: Clubs_1, Diamonds_12, etc ----
    m = re.match(r"^(Clubs|Spades|Diamonds|Hearts)_(\d+)$", s, flags=re.IGNORECASE)
    if m:
        suit = SUIT_EN[m.group(1).lower()]
        num = int(m.group(2))
        return f"{suit}_{value_code(num)}"

    # Alguns packs vêm com espaços/parenteses no nome: "Clubs_1 (Ace)" ou "Diamonds_12(queen)"
    m = re.match(r"^(Clubs|Spades|Diamonds|Hearts)_(\d+)", s, flags=re.IGNORECASE)
    if m:
        suit = SUIT_EN[m.group(1).lower()]
        num = int(m.group(2))
        return f"{suit}_{value_code(num)}"

    # ---- cartas Finlandês: hertta_01, pata_06, risti_12, ruutu_13 ----
    m = re.match(r"^(hertta|pata|risti|ruutu)_(\d+)$", low)
    if m:
        suit = SUIT_FI[m.group(1)]
        num = int(m.group(2))
        return f"{suit}_{value_code(num)}"

    # Alguns podem vir com sufixos/pastas exportadas tipo "hertta_01/png"
    # (se isso for literalmente um arquivo, não é comum; mas deixei robusto)
    m = re.match(r"^(hertta|pata|risti|ruutu)_(\d+)", low)
    if m:
        suit = SUIT_FI[m.group(1)]
        num = int(m.group(2))
        return f"{suit}_{value_code(num)}"

    return None

def unique_path(p: Path) -> Path:
    """Se já existe, gera nome com sufixo _dupN."""
    if not p.exists():
        return p
    base = p.with_suffix("")
    ext = p.suffix
    for i in range(1, 9999):
        cand = Path(f"{base}_dup{i}{ext}")
        if not cand.exists():
            return cand
    raise RuntimeError(f"Colisão demais para {p}")

def iter_files(roots: list[Path], recursive: bool):
    for root in roots:
        if root.is_file():
            yield root
            continue
        if not root.exists():
            print(f"[WARN] pasta não existe: {root}")
            continue
        if recursive:
            for dirpath, _, filenames in os.walk(root):
                for fn in filenames:
                    yield Path(dirpath) / fn
        else:
            for fn in os.listdir(root):
                yield root / fn

def main():
    parser = argparse.ArgumentParser(description="Renomeia assets de cartas para padrão c_q, s_a, etc.")
    parser.add_argument(
        "roots",
        nargs="*",
        default=[
            "/home/vodiniz/Prog/buracao-rust/buracao-web/public/assets/cards/Pixel_Cards_FREE/Sprites/",
            "/home/vodiniz/Prog/buracao-rust/buracao-web/public/assets/cards/",
        ],
        help="Pastas (ou arquivos) para processar",
    )
    parser.add_argument("--no-recursive", action="store_true", help="Não entrar em subpastas")
    parser.add_argument("--apply", action="store_true", help="Aplica (renomeia de verdade). Sem isso, é dry-run.")
    parser.add_argument("--only-ext", default="png,jpg,jpeg,webp", help="Extensões aceitas (csv), default: png,jpg,jpeg,webp")
    args = parser.parse_args()

    exts = {"." + e.strip().lower().lstrip(".") for e in args.only_ext.split(",") if e.strip()}
    roots = [Path(r).expanduser() for r in args.roots]
    recursive = not args.no_recursive
    dry_run = not args.apply

    total = 0
    renamed = 0
    skipped = 0

    for path in iter_files(roots, recursive=recursive):
        if not path.is_file():
            continue

        total += 1

        ext = path.suffix.lower()
        if ext not in exts:
            skipped += 1
            continue

        new_stem = parse_target_name(path.stem)
        if not new_stem:
            skipped += 1
            continue

        new_path = path.with_name(new_stem + ext)
        if new_path == path:
            skipped += 1
            continue

        new_path = unique_path(new_path)

        if dry_run:
            print(f"[DRY] {path}  ->  {new_path}")
        else:
            path.rename(new_path)
            print(f"[OK ] {path}  ->  {new_path}")

        renamed += 1

    print("\n--- resumo ---")
    print(f"arquivos vistos: {total}")
    print(f"renomeados:     {renamed}")
    print(f"ignorados:      {skipped}")
    print("modo:           " + ("DRY-RUN (só preview)" if dry_run else "APPLY (renomeou de verdade)"))

if __name__ == "__main__":
    main()
