#!/usr/bin/env python3
"""Cria o backlog do monk no GitHub a partir de backlog/M*.md.

Uso:
    python3 scripts/criar_issues.py --dry-run          # gera backlog/_preview/ sem tocar no GitHub
    python3 scripts/criar_issues.py                    # cria em samuka7abr/monk
    python3 scripts/criar_issues.py --repo dono/repo

Requer o GitHub CLI autenticado (`gh auth login`) com permissão de escrita no repo.
É idempotente: issues cujo título já começa com "[Mxx]" são reaproveitadas e só têm o
corpo atualizado.

Cada issue recebe no topo: responsável, aula, a lista "Depende de" e "Desbloqueia" com os
números reais (#n), e a verificação de pronto copiada do CLAUDE.md. Referências {{Mxx}} no
texto viram #n. As dependências também são registradas como "blocked by" nativas do GitHub
quando a API permite (melhor esforço).
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
BACKLOG = RAIZ / "backlog"
CLAUDE_MD = RAIZ / "CLAUDE.md"

CORES = {
    "fase:": "1d76db",
    "complexidade:alta": "b60205",
    "complexidade:media": "fbca04",
    "complexidade:baixa": "0e8a16",
    "docs": "c5def5",
    "testes": "5319e7",
    "cli": "bfdadc",
}


def ler_issue(caminho: Path) -> dict:
    texto = caminho.read_text(encoding="utf-8")
    m = re.match(r"^---\n(.*?)\n---\n(.*)$", texto, re.S)
    if not m:
        sys.exit(f"{caminho.name}: frontmatter ausente")
    meta: dict = {}
    for linha in m.group(1).splitlines():
        chave, _, valor = linha.partition(":")
        valor = valor.strip()
        if valor.startswith("["):
            meta[chave.strip()] = [v.strip() for v in valor.strip("[]").split(",") if v.strip()]
        else:
            meta[chave.strip()] = valor.strip('"')
    meta["corpo"] = m.group(2).strip()
    return meta


def verificacoes() -> dict[str, str]:
    linhas = {}
    for linha in CLAUDE_MD.read_text(encoding="utf-8").splitlines():
        m = re.match(r"^\| (M\d\d) \| (.*) \|$", linha)
        if m:
            linhas[m.group(1)] = m.group(2).replace("\\|", "|")
    return linhas


def validar(issues: dict[str, dict], verif: dict[str, str]) -> None:
    erros = []
    for iid, i in issues.items():
        for d in i["depende"]:
            if d not in issues:
                erros.append(f"{iid}: depende de {d}, que não existe")
        for ref in re.findall(r"\{\{(M\d\d)\}\}", i["corpo"]):
            if ref not in issues:
                erros.append(f"{iid}: referência {{{{{ref}}}}} inexistente")
        if iid not in verif:
            erros.append(f"{iid}: sem linha de verificação no CLAUDE.md")
    # ciclos (DFS)
    estado: dict[str, int] = {}

    def visita(n: str, pilha: list[str]) -> None:
        estado[n] = 1
        for d in issues[n]["depende"]:
            if estado.get(d) == 1:
                erros.append("ciclo: " + " -> ".join(pilha + [n, d]))
            elif d in issues and d not in estado:
                visita(d, pilha + [n])
        estado[n] = 2

    for n in issues:
        if n not in estado:
            visita(n, [])
    if erros:
        sys.exit("Backlog inválido:\n  " + "\n  ".join(erros))


def renderizar(i: dict, issues: dict[str, dict], num: dict[str, int], verif: dict[str, str]) -> str:
    def ref(iid: str) -> str:
        return f"#{num[iid]} `[{iid}]` {issues[iid]['titulo']}"

    bloqueia = [k for k, v in issues.items() if i["id"] in v["depende"]]
    partes = [
        f"**ID:** `{i['id']}` · **Responsável:** @{i['responsavel']} · **Referência:** {i['aula']}",
        "",
        "### Depende de",
    ]
    if i["depende"]:
        partes += [f"- [ ] {ref(d)}" for d in i["depende"]]
        partes.append("")
        partes.append(
            "> Esta issue assume que as issues acima já estão concluídas. Antes de começar, "
            "confira cada uma pela tabela **Verificação de cada issue** do `CLAUDE.md`."
        )
    else:
        partes.append("Nenhuma — pode começar já.")
    partes += ["", "### Desbloqueia"]
    partes += [f"- {ref(b)}" for b in bloqueia] or ["Nenhuma (fim de uma cadeia)."]
    partes += ["", "---", ""]
    corpo = re.sub(r"\{\{(M\d\d)\}\}", lambda m: f"#{num[m.group(1)]}", i["corpo"])
    partes.append(corpo)
    partes += [
        "",
        "---",
        "### Como saber que está pronta",
        verif[i["id"]],
        "",
        "Ao terminar: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` "
        "sem falhas, PR com `Closes` desta issue e revisão de outro membro.",
    ]
    return "\n".join(partes) + "\n"


def gh(*args: str, entrada: str | None = None, checar: bool = True) -> subprocess.CompletedProcess:
    r = subprocess.run(["gh", *args], input=entrada, text=True, capture_output=True)
    if checar and r.returncode != 0:
        sys.exit(f"gh {' '.join(args[:3])}… falhou:\n{r.stderr}")
    return r


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default="samuka7abr/monk")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    issues = {}
    for p in sorted(BACKLOG.glob("M*.md")):
        i = ler_issue(p)
        issues[i["id"]] = i
    verif = verificacoes()
    validar(issues, verif)
    print(f"{len(issues)} issues válidas, sem ciclos.")

    if args.dry_run:
        num = {iid: k + 1 for k, iid in enumerate(issues)}
        saida = BACKLOG / "_preview"
        saida.mkdir(exist_ok=True)
        for iid, i in issues.items():
            (saida / f"{iid}.md").write_text(
                f"# [{iid}] {i['titulo']}\n\nlabels: {', '.join(i['labels'])}\n\n"
                + renderizar(i, issues, num, verif),
                encoding="utf-8",
            )
        print(f"Prévia em {saida} (números fictícios #1..#{len(issues)}).")
        return

    repo = args.repo
    # labels
    labels = sorted({l for i in issues.values() for l in i["labels"]})
    for l in labels:
        cor = next((c for k, c in CORES.items() if l.startswith(k)), "ededed")
        gh("label", "create", l, "--repo", repo, "--color", cor, "--force")

    # issues já existentes
    existentes = json.loads(
        gh("issue", "list", "--repo", repo, "--state", "all", "--limit", "500",
           "--json", "number,title").stdout
    )
    num: dict[str, int] = {}
    for e in existentes:
        m = re.match(r"^\[(M\d\d)\]", e["title"])
        if m and m.group(1) in issues:
            num[m.group(1)] = e["number"]

    # 1ª passada: criar (corpo provisório)
    for iid, i in issues.items():
        if iid in num:
            print(f"{iid}: já existe como #{num[iid]}")
            continue
        titulo = f"[{iid}] {i['titulo']}"
        base = ["issue", "create", "--repo", repo, "--title", titulo, "--body", "(gerando…)"]
        for l in i["labels"]:
            base += ["--label", l]
        r = gh(*base, "--assignee", i["responsavel"], checar=False)
        if r.returncode != 0:
            print(f"  aviso: não consegui atribuir {iid} a @{i['responsavel']} "
                  f"(a pessoa é colaboradora do repo?). Criando sem responsável.")
            r = gh(*base)
        url = r.stdout.strip().splitlines()[-1]
        num[iid] = int(url.rstrip("/").split("/")[-1])
        print(f"{iid}: criada #{num[iid]}")

    # 2ª passada: corpo final com os números reais
    for iid, i in issues.items():
        gh("issue", "edit", str(num[iid]), "--repo", repo, "--body-file", "-",
           entrada=renderizar(i, issues, num, verif))
    print("Corpos atualizados com dependências.")

    # 3ª passada: dependências nativas (blocked by), melhor esforço
    ids_internos = {}
    falhas = 0
    for iid in issues:
        r = gh("api", f"repos/{repo}/issues/{num[iid]}", "--jq", ".id", checar=False)
        ids_internos[iid] = r.stdout.strip()
    for iid, i in issues.items():
        for d in i["depende"]:
            r = gh("api", "-X", "POST", f"repos/{repo}/issues/{num[iid]}/dependencies/blocked_by",
                   "-F", f"issue_id={ids_internos[d]}", checar=False)
            if r.returncode != 0 and "already" not in r.stderr.lower():
                falhas += 1
    if falhas:
        print(f"Aviso: {falhas} relações 'blocked by' nativas não foram registradas "
              "(a API pode não estar disponível); as dependências continuam no corpo das issues.")
    else:
        print("Relações 'blocked by' registradas.")
    print("Pronto.")


if __name__ == "__main__":
    main()
