| configuration | WER base -> variant | drug found | term found | tech found | critical samples | fixed words | broken words (all) | broken words outside key terms | extra words |
|---|---|---|---|---|---|---|---|---|---|
| sherpa-parakeet-tdt-0.6b-v3-int8 (greedy search) | 2.3 % -> 5.1 % | 4/6 -> 6/6 | 10/10 -> 9/10 | 31/34 -> 33/34 | 4 -> 7 | 3 | 30 | 29 | +3 |

#### sherpa-parakeet-tdt-0.6b-v3-int8 (greedy search)
- Fixed: `fr-adm-02-owner` expected `parties`, baseline `partis`, variant `parties`
- Fixed: `fr-it-03-owner` expected `rest`, baseline `reste`, variant `rest` (key term)
- Fixed: `fr-it-09-owner` expected `users`, baseline `user`, variant `users` (key term)
- Broken: `en-dict-01-owner` expected `12000`, baseline `12000`, variant `121000`
- Broken: `en-gen-q-01-owner` expected `what`, baseline `what`, variant `(missing)`
- Broken: `en-gen-q-01-owner` expected `time`, baseline `time`, variant `(missing)`
- Broken: `en-gen-q-01-owner` expected `is`, baseline `is`, variant `postgresql`
- Broken: `en-gen-q-01-owner` expected `my`, baseline `my`, variant `a`
- Broken: `en-gen-q-05-owner` expected `is`, baseline `is`, variant `(missing)`
- Broken: `en-gen-q-05-owner` expected `the`, baseline `the`, variant `postgresql`
- Broken: `en-gen-q-05-owner` expected `meeting`, baseline `meeting`, variant `rest`
- Broken: `en-gen-q-05-owner` expected `room`, baseline `room`, variant `api`
- Broken: `en-gen-q-05-owner` expected `free`, baseline `free`, variant `rest`
- Broken: `en-gen-q-05-owner` expected `at`, baseline `at`, variant `api`
- Broken: `en-gen-q-05-owner` expected `330`, baseline `330`, variant `30`
- Broken: `en-gen-s-02-owner` expected `the`, baseline `the`, variant `(missing)`
- Broken: `en-gen-s-02-owner` expected `letter`, baseline `letter`, variant `(missing)`
- Broken: `en-gen-s-02-owner` expected `is`, baseline `is`, variant `(missing)`
- Broken: `en-gen-s-02-owner` expected `ready`, baseline `ready`, variant `rest`
- Broken: `en-gen-s-02-owner` expected `you`, baseline `you`, variant `api`
- Broken: `en-gen-s-04-owner` expected `i`, baseline `i`, variant `(missing)`
- Broken: `en-gen-s-04-owner` expected `will`, baseline `will`, variant `(missing)`
- Broken: `en-gen-s-04-owner` expected `not`, baseline `not`, variant `rest`
- Broken: `en-gen-s-04-owner` expected `be`, baseline `be`, variant `api`
- Broken: `fr-dict-01-owner` expected `compte`, baseline `compte`, variant `contre`
- Broken: `fr-dict-01-owner` expected `artérielle`, baseline `artérielle`, variant `(missing)` (key term)
- Broken: `fr-dict-01-owner` expected `il`, baseline `il`, variant `(missing)`
- Broken: `fr-dict-01-owner` expected `ne`, baseline `ne`, variant `(missing)`
