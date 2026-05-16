# IEEE Transactions Draft (ING-INF/05)

This folder contains a full LaTeX manuscript draft based on the RETROSPECT platform.

## Files

- `main.tex`: complete paper text in IEEE Transactions style (`IEEEtran` class)
- `references.bib`: BibTeX references

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

To regenerate the paper figures from the final experiment artifact:

```bash
cd /home/ubuntu/retrospect/paper_ieee_inginf05
../.venv-paper/bin/python generate_figures.py
```

This command regenerates:

```text
figures/latency_ci_profile.png
figures/latency_boxplot.png
```

## Before Submission

1. Replace author names and affiliations in `main.tex`.
2. If you rerun experiments, update the JSON path in `generate_figures.py` and refresh the manuscript tables/figures.

## Experimental Metrics Harness

The paper now includes a measurement plan for enrollment, heartbeat supervision, disconnect recovery, application deployment, and system snapshot metrics. To execute the collector after the platform is deployed:

```bash
python3 scripts/collect_experiment_metrics.py \
	--api-base http://127.0.0.1:3001 \
	--gateway-http http://127.0.0.1:8080 \
	--namespace wasmbed \
	--trials 30
```

The script writes a JSON file under `experiments/` with raw trial records and aggregated summaries. The manuscript now reports mean, standard deviation, median, p90, p95, p99, interquartile range, coefficient of variation, 95% confidence interval for latency metrics, Wilson intervals for binary success rates, and transactional end-to-end metrics derived from per-trial composition.
3. Check target journal rules (page limit, graphical abstract, data availability statement).
4. Run final language and formatting pass.
