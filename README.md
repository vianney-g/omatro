# omatro

Un jeu de cartes à la Balatro pour Omarchy : on joue des mains de poker pour battre des
blinds, et des jokers multiplient le score (jetons × mult).

- `engine/` : le moteur en Rust (règles, score, jokers), un processus qui dialogue en JSON.
- `plugin/` : l'affichage, en plugin Quickshell (panneau avec sa propre fenêtre) pour le
  shell Omarchy, aux couleurs et dans la police du thème actif.

Projet personnel, sans lien avec Balatro : les mécaniques sont reprises, mais ni le nom,
ni les graphismes, ni les textes, ni le code.

## Installation

```sh
make install   # compile le moteur, le copie dans plugin/bin, relie et active le plugin
omarchy-shell shell toggle io.github.vianney-g.omatro '{}'
```

Après une modification du moteur : `make plugin`, puis `omarchy restart shell` (le plugin
reste chargé entre deux ouvertures et le shell ne suit pas le lien symbolique).

Touches : 1 à 8 pour choisir les cartes, Entrée pour jouer, D pour défausser,
N pour une nouvelle partie, Échap pour fermer.

## Moteur

```sh
cd engine
cargo test
cargo run      # lit les commandes sur stdin
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
| `{"cmd":"next_blind"}` | Passe à la blind suivante, une fois la blind gagnée |

Réponses :

- `{"event":"state","state":{…}}` : état de la partie (blind, objectif, score, mains et
  défausses restantes, `phase` = `playing` / `won` / `lost`, main, jokers).
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

| Id | Nom | Effet |
|---|---|---|
| `baobab` | Baobab | +30 jetons |
| `marche` | Marché | +2 mult pour chaque carreau qui marque |
| `griot` | Griot | +20 jetons pour chaque figure qui marque |
| `harmattan` | Harmattan | ×2 mult si la main jouée compte 3 cartes ou moins |
