# omatro

Un jeu de cartes à la Balatro pour Omarchy : on joue des mains de poker pour battre des
blinds, et des jokers multiplient le score (jetons × mult).

Une application Qt Quick autonome, comme omawrite ou omacut, mais écrite en Rust
(liaison Qt avec [cxx-qt](https://github.com/KDAB/cxx-qt)) :

- `engine/` : le moteur (règles, score, jokers, boutique), sans dépendance à Qt. Il
  fournit aussi un binaire `omatro-engine` qui parle le protocole JSON sur stdin/stdout.
- `app/` : l'application. Le QML est embarqué dans le binaire, et le moteur est appelé
  directement, avec le même protocole JSON. Le thème Omarchy actif (couleurs de
  `colors.toml`, taille de police de `shell.toml`, arrondi Hyprland) est relu dès qu'il
  change.
- `pkgbuild/` : entrée du lanceur et icône.

Projet personnel, sans lien avec Balatro : les mécaniques sont reprises, mais ni le nom,
ni les graphismes, ni les textes, ni le code.

## Installation

Il faut Rust et Qt 6 (`qt6-base`, `qt6-declarative`).

```sh
make install   # compile et installe dans ~/.local (binaire, lanceur, icône)
omatro
make run       # compile et lance sans installer
make test
```

Touches : 1 à 8 pour choisir les cartes, Entrée pour jouer, D pour défausser,
N pour une nouvelle partie, Ctrl+Q pour quitter. En boutique : 1 et 2 pour acheter,
R pour relancer, Entrée pour la blind suivante.

## Déroulement

Chaque blind demande d'atteindre un score (300, puis × 1,5) en 4 mains et 3 défausses.
Une blind gagnée rapporte 4 $, plus 1 $ par main non jouée, plus 1 $ d'intérêt par
tranche de 5 $ possédés (5 $ au plus). On encaisse, puis la boutique propose 2 jokers
qu'on n'a pas encore (5 emplacements, revente à moitié prix ; relance à 5 $, puis 1 $ de
plus à chaque fois). On commence avec 4 $ et sans joker.

## Moteur seul

```sh
cargo run -p omatro-engine   # lit les commandes sur stdin
```

## Protocole

Une commande JSON par ligne sur l'entrée standard, une réponse JSON par ligne sur la sortie.

| Commande | Effet |
|---|---|
| `{"cmd":"new_game","seed":42,"jokers":["baobab"]}` | Nouvelle partie (`seed` et `jokers` facultatifs) |
| `{"cmd":"state"}` | Renvoie l'état |
| `{"cmd":"play","cards":[0,2,3]}` | Joue 1 à 5 cartes (indices dans la main) |
| `{"cmd":"discard","cards":[1]}` | Défausse 1 à 5 cartes |
| `{"cmd":"preview","cards":[0,2]}` | Main que formerait la sélection, sans la jouer |
| `{"cmd":"cash_out"}` | Encaisse la récompense d'une blind gagnée et ouvre la boutique |
| `{"cmd":"buy","offer":0}` | Achète une offre de la boutique |
| `{"cmd":"sell","joker":0}` | Revend un joker (en boutique) |
| `{"cmd":"reroll"}` | Relance les offres de la boutique |
| `{"cmd":"next_blind"}` | Quitte la boutique pour la blind suivante |

Réponses :

- `{"event":"state","state":{…}}` : état de la partie (blind, objectif, score, mains et
  défausses restantes, `phase` = `playing` / `won` / `shop` / `lost`, main, jokers,
  `money`, `reward` en phase `won`, offres `shop` et `reroll_cost` en boutique).
- `{"event":"scored","score":{…},"state":{…}}` après `play` : cartes jouées, main reconnue,
  cartes qui marquent, jetons et mult de départ, puis `steps` (chaque déclencheur, avec
  jetons et mult après coup) pour animer le calcul, et le `total`.
- `{"event":"preview","cards":[…],"hand":"pair","scoring":[…],"base_chips":10,"base_mult":2.0}`
  (`hand` à `null` si rien n'est choisi, `scoring` en indices dans la main).
- `{"event":"error","message":"…"}`.

La main est toujours rangée par rang décroissant.

Les cartes s'écrivent `{"rank":14,"suit":"spades"}` (rang 2 à 14 : 11 valet, 12 dame,
13 roi, 14 as).

## Jokers

| Id | Nom | Prix | Effet |
|---|---|---|---|
| `baobab` | Baobab | 4 $ | +30 jetons |
| `calebasse` | Calebasse | 4 $ | +5 mult |
| `marche` | Marché | 5 $ | +2 mult pour chaque carreau qui marque |
| `pagne` | Pagne | 5 $ | +15 jetons pour chaque cœur qui marque |
| `griot` | Griot | 5 $ | +20 jetons pour chaque figure qui marque |
| `zemidjan` | Zémidjan | 5 $ | +12 mult si la main est une suite |
| `tontine` | Tontine | 5 $ | +1 $ par défausse restante à la fin de la blind |
| `djembe` | Djembé | 6 $ | +2 mult pour chaque carte qui marque |
| `harmattan` | Harmattan | 7 $ | ×2 mult si la main jouée compte 3 cartes ou moins |
| `masque` | Masque | 8 $ | ×3 mult si la main est un brelan |
