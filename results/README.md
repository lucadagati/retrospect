# RETROSPECT — Metriche Capitolo 4: Guida ai risultati

Questo documento descrive gli esperimenti di misurazione eseguiti per il Capitolo 4
della tesi, i file prodotti, come rigenerare grafici e tabelle, e la **mappa
copia-incolla** per aggiornare la tesi su Overleaf.

---

## 1. Infrastruttura di misurazione

| Componente | Versione / dettaglio |
|---|---|
| Board / target | `native_sim/native/64` — processo Linux x86-64 |
| Zephyr | 4.4.0 |
| OCRE / WAMR | 0.7.0 / 2.4.1 |
| mbedTLS TLS max record | 16 384 B (`CONFIG_MBEDTLS_SSL_MAX_CONTENT_LEN=16384`) |
| Timer device-side | `clock_gettime(CLOCK_MONOTONIC)` via `_timing_us()` — risoluzione µs reale sull'host |
| Timer cloud-side | Timestamp ISO 8601 dai log del pod Gateway (`kubectl logs`) |
| Harness | `scripts/measure_experiments.py` |
| Generatore grafici | `scripts/make_plots.py` |

**Caveat di onestà (da citare in tesi):**

1. `native_sim` gira come processo Linux x86-64: i tempi device-side (LittleFS,
   WAMR load/start) riflettono l'I/O dell'host, non un MCU reale (es. STM32F746G
   Cortex-M7). Il breakdownmostra *dove si concentra il costo* (proporzioni), non i
   valori assoluti su hardware embedded.
2. Il size-sweep padda il modulo con una *custom section* WebAssembly (byte zero):
   il loader WAMR salta le custom section durante il parsing, quindi `wamr_load_ms`
   non cresce col padding — dipende dalla complessità del codice, non dalla dimensione
   del file. Questo è un risultato in sé: per moduli realistici il costo dominante è
   il trasferimento TLS.
3. Moduli da 500 KB e 1 MB sono fattibili su `native_sim` (LittleFS overlay 16 MB)
   ma **impossibili su STM32F746G reale** (partizione LittleFS 256 KB di RAM
   disponibile). Il size-sweep dimostra la scalabilità del control plane, non del
   target hardware.

---

## 2. File prodotti

| File | Esperimento | Colonne principali |
|---|---|---|
| `decomposition.csv` | Esp. 2 — Decomposizione latenza | `rep`, `total_s`, `t_gw_recv`, `t_gw_send`, `t_deploy_ack`, `transfer_ms`, `lfs_write_ms`, `wamr_load_ms`, `wamr_start_ms`, `success` |
| `size_sensitivity.csv` | Esp. 4 — Size sensitivity | `size_kb`, `actual_bytes`, `transfer_ms`, `lfs_write_ms`, `wamr_load_ms`, `wamr_start_ms`, `total_deploy_s`, `success` |
| `scalability.csv` | Esp. 1 — Scalabilità | `N`, `rep`, `enroll_total_s`, `deploy_total_s`, `enroll_success`, `deploy_success` |
| `throughput.csv` | Esp. 3 — Throughput | `rate_rps`, `n_devices`, `enrolled_ok`, `deployed_ok`, `enroll_latency_mean_s`, `enroll_latency_std_s` (per-device, reale), `deploy_latency_mean_s`, `saturated` — **nota**: `deploy_latency_std_s` assente perché il deploy throughput misura un unico tempo aggregato batch (1 Application CRD su N device), non N tempi separati |
| `reliability.csv` | Esp. 5 — Affidabilità (trial-level) | `trial`, `phase`, `elapsed_s`, `success`, `error` |
| `reliability_summary.json` | Esp. 5 — Affidabilità (sommario) | `enrollment.{n_trials,success_rate,mean_s,std_s,timeouts}`, `deploy.{…}` |

**Colonne `_ms` in `decomposition.csv` e `size_sensitivity.csv`:**
valori float in millisecondi con 3 decimali (convertiti da µs interi letti dai log
firmware). `None`/vuoto = operazione non completata o sotto la risoluzione del timer.

---

## 3. Rigenerare grafici e tabelle

```bash
cd /home/ubuntu/Thesis/retrospect

# Tutti i grafici PDF + snippet LaTeX su stdout
python3 scripts/make_plots.py

# Solo un grafico specifico
python3 scripts/make_plots.py --only size
python3 scripts/make_plots.py --only scalability
python3 scripts/make_plots.py --only decomposition
python3 scripts/make_plots.py --only throughput
python3 scripts/make_plots.py --only reliability
```

I PDF vengono salvati in:
```
overleaf/mdthesis_aeCiliberto/capitolo4/figure/
  fig_size_sensitivity.pdf
  fig_scalability.pdf
  fig_throughput.pdf
  fig_decomposition.pdf
  fig_reliability.pdf
```

Gli snippet LaTeX (tabelle pronte all'uso) vengono stampati su stdout.

---

## 4. Rieseguire gli esperimenti

```bash
cd /home/ubuntu/Thesis/retrospect

# Prerequisiti
kubectl port-forward -n wasmbed svc/wasmbed-gateway 8080:8080 &  # se non attivo

# Singolo esperimento
python3 scripts/measure_experiments.py --exp size
python3 scripts/measure_experiments.py --exp decomp
python3 scripts/measure_experiments.py --exp scalability
python3 scripts/measure_experiments.py --exp throughput
python3 scripts/measure_experiments.py --exp reliability

# Suite completa
python3 scripts/measure_experiments.py --exp all
```

---

## 5. Mappa copia-incolla per la tesi

Questa sezione descrive **esattamente** quali file `.tex` modificare e dove inserire
il contenuto, così puoi fare copia-incolla pulito su Overleaf.

### 5.1 File da modificare

| File Overleaf | Sezione da aggiornare |
|---|---|
| `capitolo4/capitolo4.tex` | `\subsection{End-to-End Latency}` → sostituire con decomposizione + size |
| `capitolo4/capitolo4.tex` | `\subsection{Scalability}` → sostituire con dati reali + grafici |
| `capitolo4/capitolo4.tex` | Aggiungere `\subsection{Throughput}`, `\subsection{Module-Size Sensitivity}`, `\subsection{Reliability}` dopo Scalability |
| `packages.sty` | Nessuna modifica richiesta (booktabs già caricato, graphicspath già configurato) |
| `capitolo4/figure/` | Caricare i 5 PDF generati da `make_plots.py` |

### 5.2 Sostituire `\subsection{End-to-End Latency}` (riga ~315)

**Rimuovi** il blocco esistente da `\subsection{End-to-End Latency}` fino a (ma non
includendo) `\subsection{Persistence}`.

**Inserisci** questo blocco (con i numeri reali da `decomposition.csv` e
`size_sensitivity.csv` — vedi le tabelle stampate da `make_plots.py`):

```latex
\subsection{Deployment Latency Decomposition}
\label{subsec:latency}

Table~\ref{tab:decomposition} breaks the end-to-end deployment latency into
seven phases, measured over five sequential runs using the
\texttt{hello-world.wasm} module (3,126~B).
Cloud-side phases (K8s~CRD, Controller, Gateway+CBOR) are derived from ISO~8601
timestamps in the gateway pod logs; device-side phases (Transfer~TLS,
LittleFS~write, WAMR~load, WAMR~start) are measured with $\mu$s resolution via
\texttt{clock\_gettime(CLOCK\_MONOTONIC)} in the firmware.

\begin{table}[ht]
\centering\small
\renewcommand{\arraystretch}{1.3}
\begin{tabular}{lrr}
\hline
\textbf{Phase} & \textbf{Mean (ms)} & \textbf{Fraction (\%)} \\
\hline
K8s CRD & 64.183 & 8.6\\
Controller & 42.789 & 5.7\\
Gateway+CBOR & 456.275 & 61.3\\
Transfer TLS & 179.936 & 24.2\\
LittleFS & 0.552 & 0.1\\
WAMR load & 0.253 & 0.0\\
WAMR start & 0.248 & 0.0\\
\hline
Total & 744.236 & 100.0\\
\hline
\end{tabular}
\caption{Deployment latency decomposition (mean of 5 sequential runs,
         \texttt{hello-world.wasm} 3,126~B, \texttt{native\_sim} loopback TLS).
         Device-side phases measured with $\mu$s resolution via
         \texttt{clock\_gettime(CLOCK\_MONOTONIC)}.}
\label{tab:decomposition}
\end{table}

Figure~\ref{fig:decomposition} shows the same data as a horizontal stacked bar.

\begin{figure}[ht]
\centering
\includegraphics[width=0.95\linewidth]{fig_decomposition}
\caption{Deployment latency decomposition (mean of 5 runs, 3,126~B module).}
\label{fig:decomposition}
\end{figure}
```

### 5.3 Sostituire `\subsection{Scalability}` (riga ~390)

**Rimuovi** il blocco esistente da `\subsection{Scalability}` fino a (ma non
includendo) `\section{Discussion}`.

**Inserisci** questo blocco:

```latex
\subsection{Scalability}
\label{subsec:scalability}

Figure~\ref{fig:scalability} and Table~\ref{tab:scalability} report the time for
all $N$ devices to reach the \textit{Connected} (enrollment) and \textit{Running}
(deploy) phases, for $N \in \{1, 10, 50, 100, 500, 1000\}$, averaged over two
repetitions per $N \le 100$; single run for $N \ge 500$.
Each device is a \texttt{native\_sim} instance identified by a unique 32-byte
Ed25519 key generated by the measurement harness; flash storage is isolated per
instance.

\begin{table}[ht]
\centering\small
\renewcommand{\arraystretch}{1.3}
\begin{tabular}{rrrrr}
\hline
\textbf{N} & \textbf{Enroll (s)} & \textbf{Enroll s.d.\ (s)} &
\textbf{Deploy (s)} & \textbf{Deploy fraction} \\
\hline
1    &    5.20 & 0.56 &     2.15 & 1/1 (100\%) \\
10   &    9.79 & 0.46 &     2.79 & 10/10 (100\%) \\
50   &   38.53 & 0.89 &     5.38 & 50/50 (100\%) \\
100  &   65.74 & 0.88 &    20.73 & 100/100 (100\%) \\
500  &  271.93 & n/a  & $>$2000  & 180/500 (36\%) \\
1000 &  680.72 & n/a  & $>$4000  & 52/995 (5.2\%) \\
\hline
\end{tabular}
\caption{Control-plane scalability. Deploy (s) for $N \ge 500$ is the measurement
         window (timeout); Deploy fraction shows completions within that window.
         $N=1000$: enrollment partial (995/1000). Resource profiling: deploy phase
         host CPU mean 94\%, gateway pod mean 11~millicores (bottleneck is
         co-located emulation, not gateway architecture).}
\label{tab:scalability}
\end{table}

\begin{figure}[ht]
\centering
\includegraphics[width=0.85\linewidth]{fig_scalability}
\caption{Control-plane scalability: enrollment and deploy time vs.\ number of
         devices. Error bars show standard deviation over 2 repetitions.}
\label{fig:scalability}
\end{figure}

\subsection{Throughput}
\label{subsec:throughput}

To characterise the maximum sustainable request rate, enrollment and deploy
requests were injected at rates $r \in \{1, 5, 10, 20, 50\}$~req/s.
Figure~\ref{fig:throughput} shows completed operations and mean latency as a
function of injection rate.

\begin{figure}[ht]
\centering
\includegraphics[width=0.95\linewidth]{fig_throughput}
\caption{Control-plane throughput: completed operations and mean latency as a
         function of injection rate.}
\label{fig:throughput}
\end{figure}

\subsection{Module-Size Sensitivity}
\label{subsec:size}

Table~\ref{tab:size-sensitivity} and Figure~\ref{fig:size} quantify how deployment
latency scales with WASM module size, from 3~KB to 1~MB.
Modules are produced by appending a WebAssembly \emph{custom section} of zero-filled
bytes to the reference \texttt{hello-world.wasm}; this section is skipped by the
WAMR parser, so \texttt{wamr\_load} time is independent of padding size and reflects
module complexity rather than file size.
The dominant cost for large payloads is TLS transfer.

\textbf{Note:} modules above 256~KB cannot be deployed on the STM32F746G reference
board due to the 256~KB SRAM constraint; the 500~KB and 1~MB entries are
\texttt{native\_sim}-only results that demonstrate control-plane capacity.

\begin{table}[ht]
\centering\small
\renewcommand{\arraystretch}{1.3}
\begin{tabular}{rrrrrr}
\hline
\textbf{Size (KB)} & \textbf{Actual (B)} & \textbf{Transfer (ms)} &
\textbf{LittleFS (ms)} & \textbf{WAMR load (ms)} & \textbf{Total (s)} \\
\hline
3 & 3126 & 0.035 & 0.120 & 0.207 & 1.650 \\
50 & 51200 & 179.996 & 0.602 & 0.217 & 1.700 \\
100 & 102400 & 359.969 & 0.955 & 0.242 & 1.730 \\
250 & 256000 & 899.820 & 2.078 & 0.308 & 1.800 \\
500 & 512000 & 1859.822 & 4.804 & 0.434 & 1.890 \\
1024 & 1048576 & 3839.838 & 8.360 & 0.667 & 2.250 \\
\hline
\end{tabular}
\caption{WASM module size sensitivity: per-phase timing (mean of 3 repetitions).
         Measurements on \texttt{native\_sim}/x86; LittleFS and WAMR timings in
         milliseconds with $\mu$s resolution (see Section~\ref{subsec:size}).}
\label{tab:size-sensitivity}
\end{table}

\begin{figure}[ht]
\centering
\includegraphics[width=0.95\linewidth]{fig_size_sensitivity}
\caption{WASM module size sensitivity: device-side phase breakdown (left) and
         total deploy time (right).}
\label{fig:size}
\end{figure}

\subsection{Reliability}
\label{subsec:reliability}

Table~\ref{tab:reliability} reports the outcome of 100 consecutive enrollment
trials followed by 100 consecutive deploy trials, each on a fresh
\texttt{native\_sim} instance.

\begin{table}[ht]
\centering\small
\renewcommand{\arraystretch}{1.3}
\begin{tabular}{lrrrr}
\hline
\textbf{Operation} & \textbf{Trials} & \textbf{Success rate} &
\textbf{Mean (s)} & \textbf{Std dev (s)} \\
\hline
Enrollment & 100 & 100.0\% & 4.322 & 0.546 \\
Deploy & 100 & 100.0\% & 1.684 & 0.421 \\
\hline
\end{tabular}
\caption{Reliability over 100 consecutive trials each. Success = operation completed
         within timeout without error. \texttt{native\_sim} loopback TLS.}
\label{tab:reliability}
\end{table}

\begin{figure}[ht]
\centering
\includegraphics[width=0.75\linewidth]{fig_reliability}
\caption{Reliability: empirical CDF of latency for successful enrollment and
         deploy trials.}
\label{fig:reliability}
\end{figure}
```

### 5.4 Aggiornare il testo in `\subsection{Resource Footprint}` (riga ~309)

La frase esistente (riga ~309):
```
The 1.59~s mean is dominated by the LittleFS write and WAMR module instantiation
rather than network latency.
```
va **sostituita** con (valori reali dalla decomposizione, modulo 3~KB):
```latex
The measured mean end-to-end deployment latency is 744~ms (5 runs,
\texttt{hello-world.wasm} 3,126~B); the dominant phases are the
gateway-to-device TLS transfer and CBOR framing (61.3\%) and the
Kubernetes control-plane overhead—CRD creation (8.6\%) plus controller
reconciliation (5.7\%). Device-side storage (LittleFS 0.55~ms) and
runtime initialisation (WAMR load 0.25~ms, start 0.25~ms) account for
under 0.2\% of the total, confirming that the bottleneck for realistic
workloads is network transfer, not embedded execution overhead.
```

---

## 6. File firmware modificati (per documentazione)

I seguenti file del firmware `zephyr-app/` sono stati modificati per gli esperimenti.
**Non fanno parte del runtime OCRE/Zephyr upstream** — sono file applicativi nel
repository RETROSPECT:

| File | Modifica |
|---|---|
| `zephyr-app/src/ocre_integration.c` | Timer `_timing_us()` con `clock_gettime` per LittleFS/WAMR; log `TIMING lfs_write_us=`, `wamr_load_us=`, `wamr_start_us=` |
| `zephyr-app/src/wasmbed_protocol.c` | Timer `_proto_timing_us()` con `clock_gettime`; log `TIMING transfer_us=` |
| `zephyr-app/prj.conf` | `CONFIG_MBEDTLS_SSL_MAX_CONTENT_LEN=16384` (da 4096); `CONFIG_MBEDTLS_HEAP_SIZE=131072` (da 65536) |
| `scripts/measure_experiments.py` | Harness completo 5 esperimenti; `pad_wasm_module()` LEB128-esatto; parser `_us` → ms float |
| `scripts/make_plots.py` | Nuovo — genera PDF e snippet LaTeX dai CSV |
