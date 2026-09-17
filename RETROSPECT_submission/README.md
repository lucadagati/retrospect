# Computer Communications CAS Submission Draft

This folder contains a LaTeX manuscript draft prepared for the Computer Communications special issue "Sustainable Digital Research Infrastructures for the Edge-to-Cloud Continuum".

## Files

- `main.tex`: complete paper in Elsevier CAS double-column style (`cas-dc`).
- `cas-dc.cls`: thin wrapper that loads the local Elsevier CAS class from `els-cas-templates/`.
- `els-cas-templates/`: local CAS template support files required by the manuscript.
- `figures/`: PNG plots referenced by `\includegraphics` (upload this folder to Overleaf).
- `sections/`: modular section files (problem → architecture → protocol → implementation → evaluation → discussion).
- `references.bib`: BibTeX references.
- `cover_letter.md`: cover letter customized for `VSI: Sustainable Digital Research Infrastructures`.

## Build

If `latexmk` is available:

```bash
latexmk -pdf main.tex
```

Alternative with `pdflatex` + `bibtex`:

```bash
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```

Build PDF:

```bash
cd RETROSPECT_submission && latexmk -pdf main.tex
```

## Before Submission

1. Confirm corresponding author details in `main.tex`.
2. Replace placeholder contact details in `cover_letter.md`.
3. Check the final Computer Communications Guide for Authors requirements.
4. Review highlights, declarations, figures, and references before upload.
