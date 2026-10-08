| configuration | WER base -> variant | drug found | term found | tech found | critical samples | fixed words | broken words (all) | broken words outside key terms | extra words |
|---|---|---|---|---|---|---|---|---|---|
| sherpa-parakeet-tdt-0.6b-v3-int8 (greedy search) | 2.3 % -> 2.5 % | 4/6 -> 6/6 | 10/10 -> 10/10 | 31/34 -> 31/34 | 4 -> 2 | 2 | 3 | 3 | +1 |

#### sherpa-parakeet-tdt-0.6b-v3-int8 (greedy search)
- Fixed: `fr-adm-02-owner` expected `parties`, baseline `partis`, variant `parties`
- Fixed: `fr-dict-01-owner` expected `d'amoxicilline`, baseline `d'amoxycilline`, variant `d'amoxicilline` (key term)
- Broken: `fr-dict-01-owner` expected `il`, baseline `il`, variant `(missing)`
- Broken: `fr-dict-01-owner` expected `ne`, baseline `ne`, variant `(missing)`
- Broken: `fr-dict-01-owner` expected `signale`, baseline `signale`, variant `(missing)`
