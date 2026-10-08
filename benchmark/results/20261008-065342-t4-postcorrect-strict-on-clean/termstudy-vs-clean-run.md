| configuration | WER base -> variant | drug found | term found | tech found | critical samples | fixed words | broken words (all) | broken words outside key terms | extra words |
|---|---|---|---|---|---|---|---|---|---|
| sherpa-canary-180m-flash-int8 (greedy search) | 5.9 % -> 5.5 % | 4/6 -> 5/6 | 9/10 -> 9/10 | 21/34 -> 23/34 | 3 -> 2 | 3 | 0 | 0 | -1 |
| sherpa-parakeet-tdt-0.6b-v3-int8 (greedy search) | 2.3 % -> 2.1 % | 4/6 -> 6/6 | 10/10 -> 10/10 | 31/34 -> 31/34 | 4 -> 2 | 2 | 0 | 0 | +0 |
| sherpa-whisper-tiny (greedy search) | 25.4 % -> 24.8 % | 1/6 -> 2/6 | 4/10 -> 5/10 | 16/34 -> 18/34 | 19 -> 19 | 5 | 0 | 0 | -2 |
| whisper-cpp-base-q5_1 (beam search (5 beams)) | 8.4 % -> 7.6 % | 3/6 -> 5/6 | 5/10 -> 8/10 | 24/34 -> 25/34 | 12 -> 10 | 7 | 0 | 0 | -2 |
| whisper-cpp-base-q5_1 (greedy (beam 1)) | 11.4 % -> 10.4 % | 2/6 -> 4/6 | 5/10 -> 8/10 | 20/34 -> 21/34 | 15 -> 13 | 9 | 0 | 0 | -1 |
| whisper-cpp-small-q5_1 (beam search (5 beams)) | 3.9 % -> 3.5 % | 3/6 -> 5/6 | 9/10 -> 10/10 | 28/34 -> 28/34 | 7 -> 5 | 4 | 0 | 0 | +0 |
| whisper-cpp-small-q5_1 (greedy (beam 1)) | 4.5 % -> 3.9 % | 3/6 -> 5/6 | 8/10 -> 10/10 | 27/34 -> 28/34 | 7 -> 5 | 6 | 0 | 0 | +0 |
| whisper-cpp-tiny (beam search (5 beams)) | 16.1 % -> 14.7 % | 1/6 -> 3/6 | 5/10 -> 6/10 | 19/34 -> 23/34 | 13 -> 12 | 11 | 0 | 0 | -4 |
| whisper-cpp-tiny (greedy (beam 1)) | 19.8 % -> 19.4 % | 2/6 -> 2/6 | 4/10 -> 4/10 | 17/34 -> 19/34 | 18 -> 18 | 3 | 0 | 0 | -1 |

#### sherpa-canary-180m-flash-int8 (greedy search)
- Fixed: `fr-it-03-owner` expected `rest`, baseline `reste`, variant `rest` (key term)
- Fixed: `fr-it-08-owner` expected `backup`, baseline `up`, variant `backup` (key term)
- Fixed: `fr-med-01-owner` expected `d'amoxicilline`, baseline `d'amoxiciline`, variant `d'amoxicilline` (key term)

#### sherpa-parakeet-tdt-0.6b-v3-int8 (greedy search)
- Fixed: `fr-dict-01-owner` expected `d'amoxicilline`, baseline `d'amoxycilline`, variant `d'amoxicilline` (key term)
- Fixed: `fr-med-01-owner` expected `d'amoxicilline`, baseline `d'amoxycilline`, variant `d'amoxicilline` (key term)

#### sherpa-whisper-tiny (greedy search)
- Fixed: `en-it-05-owner` expected `firewall`, baseline `wall`, variant `firewall` (key term)
- Fixed: `fr-dict-01-owner` expected `d'amoxicilline`, baseline `d'amoxiciline`, variant `d'amoxicilline` (key term)
- Fixed: `fr-it-05-owner` expected `l'authentification`, baseline `l'autantification`, variant `l'authentification` (key term)
- Fixed: `fr-leg-07-owner` expected `tacite`, baseline `tassite`, variant `tacite` (key term)
- Fixed: `fr-med-03-owner` expected `hypertension`, baseline `tension`, variant `hypertension` (key term)

#### whisper-cpp-base-q5_1 (beam search (5 beams))
- Fixed: `fr-it-07-owner` expected `firewall`, baseline `wall`, variant `firewall` (key term)
- Fixed: `fr-med-01-owner` expected `d'amoxicilline`, baseline `d'amoxiciline`, variant `d'amoxicilline` (key term)
- Fixed: `fr-med-03-owner` expected `hypertension`, baseline `hipertension`, variant `hypertension` (key term)
- Fixed: `fr-med-04-owner` expected `paracétamol`, baseline `sétamol`, variant `paracétamol` (key term)
- Fixed: `fr-med-05-owner` expected `contre`, baseline `(missing)`, variant `contre` (key term)
- Fixed: `fr-med-05-owner` expected `indiqué`, baseline `contraindiqué`, variant `indiqué` (key term)
- Fixed: `fr-med-10-owner` expected `rénale`, baseline `rénal`, variant `rénale` (key term)

#### whisper-cpp-base-q5_1 (greedy (beam 1))
- Fixed: `fr-it-07-owner` expected `firewall`, baseline `wall`, variant `firewall` (key term)
- Fixed: `fr-leg-03-owner` expected `confidentialité`, baseline `confidenciélité`, variant `confidentialité` (key term)
- Fixed: `fr-med-02-owner` expected `pénicilline`, baseline `péniciline`, variant `pénicilline` (key term)
- Fixed: `fr-med-03-owner` expected `hypertension`, baseline `hipertension`, variant `hypertension` (key term)
- Fixed: `fr-med-04-owner` expected `paracétamol`, baseline `sétamol`, variant `paracétamol` (key term)
- Fixed: `fr-med-05-owner` expected `est`, baseline `oigulant`, variant `est`
- Fixed: `fr-med-05-owner` expected `contre`, baseline `est`, variant `contre` (key term)
- Fixed: `fr-med-05-owner` expected `indiqué`, baseline `contraindiquée`, variant `indiqué` (key term)
- Fixed: `fr-med-10-owner` expected `rénale`, baseline `rénal`, variant `rénale` (key term)

#### whisper-cpp-small-q5_1 (beam search (5 beams))
- Fixed: `fr-dict-01-owner` expected `d'amoxicilline`, baseline `d'amoxiciline`, variant `d'amoxicilline` (key term)
- Fixed: `fr-med-01-owner` expected `d'amoxicilline`, baseline `d'amoxiciline`, variant `d'amoxicilline` (key term)
- Fixed: `fr-med-05-owner` expected `contre`, baseline `(missing)`, variant `contre` (key term)
- Fixed: `fr-med-05-owner` expected `indiqué`, baseline `contraindiqué`, variant `indiqué` (key term)

#### whisper-cpp-small-q5_1 (greedy (beam 1))
- Fixed: `fr-dict-01-owner` expected `d'amoxicilline`, baseline `d'amoxiciline`, variant `d'amoxicilline` (key term)
- Fixed: `fr-it-12-owner` expected `docker`, baseline `dockeur`, variant `docker` (key term)
- Fixed: `fr-med-01-owner` expected `d'amoxicilline`, baseline `d'amoxiciline`, variant `d'amoxicilline` (key term)
- Fixed: `fr-med-05-owner` expected `contre`, baseline `(missing)`, variant `contre` (key term)
- Fixed: `fr-med-05-owner` expected `indiqué`, baseline `contreindiqué`, variant `indiqué` (key term)
- Fixed: `fr-med-10-owner` expected `rénale`, baseline `renale`, variant `rénale` (key term)

#### whisper-cpp-tiny (beam search (5 beams))
- Fixed: `en-it-05-owner` expected `firewall`, baseline `wall`, variant `firewall` (key term)
- Fixed: `fr-dict-01-owner` expected `d'amoxicilline`, baseline `d'amoxiciline`, variant `d'amoxicilline` (key term)
- Fixed: `fr-it-01-owner` expected `postgresql`, baseline `gresql`, variant `postgresql` (key term)
- Fixed: `fr-it-05-owner` expected `l'authentification`, baseline `l'autantification`, variant `l'authentification` (key term)
- Fixed: `fr-it-08-owner` expected `backup`, baseline `up`, variant `backup` (key term)
- Fixed: `fr-leg-07-owner` expected `tacite`, baseline `tassite`, variant `tacite` (key term)
- Fixed: `fr-med-03-owner` expected `artérielle`, baseline `arterielle`, variant `artérielle` (key term)
- Fixed: `fr-med-03-owner` expected `hypertension`, baseline `tension`, variant `hypertension` (key term)
- Fixed: `fr-med-03-owner` expected `artérielle`, baseline `arterielle`, variant `artérielle` (key term)
- Fixed: `fr-med-05-owner` expected `l'anticoagulant`, baseline `l'anti`, variant `l'anticoagulant` (key term)
- Fixed: `fr-med-05-owner` expected `est`, baseline `coagulant`, variant `est`

#### whisper-cpp-tiny (greedy (beam 1))
- Fixed: `en-it-05-owner` expected `firewall`, baseline `wall`, variant `firewall` (key term)
- Fixed: `fr-it-05-owner` expected `l'authentification`, baseline `l'autantification`, variant `l'authentification` (key term)
- Fixed: `fr-leg-07-owner` expected `tacite`, baseline `tassite`, variant `tacite` (key term)
