# omatro

Un jeu de cartes à la Balatro pour Omarchy : on joue des mains de poker pour battre des
blinds, et des jokers multiplient le score (jetons × mult).

- `engine/` : le moteur en Rust (règles, score, jokers), un processus qui dialogue en JSON.
- `plugin/` (à venir) : l'affichage, en plugin Quickshell pour le shell Omarchy, aux
  couleurs du thème actif.

Projet personnel, sans lien avec Balatro : les mécaniques sont reprises, mais ni le nom,
ni les graphismes, ni les textes, ni le code.

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
| `{"cmd":"next_blind"}` | Passe à la blind suivante, une fois la blind gagnée |

Réponses :

- `{"event":"state","state":{…}}` : état de la partie (blind, objectif, score, mains et
  défausses restantes, `phase` = `playing` / `won` / `lost`, main, jokers).
- `{"event":"scored","score":{…},"state":{…}}` après `play` : cartes jouées, main reconnue,
  cartes qui marquent, jetons et mult de départ, puis `steps` (chaque déclencheur, avec
  jetons et mult après coup) pour animer le calcul, et le `total`.
- `{"event":"error","message":"…"}`.

Les cartes s'écrivent `{"rank":14,"suit":"spades"}` (rang 2 à 14 : 11 valet, 12 dame,
13 roi, 14 as).

## Jokers

| Id | Nom | Effet |
|---|---|---|
| `baobab` | Baobab | +30 jetons |
| `marche` | Marché | +2 mult pour chaque carreau qui marque |
| `griot` | Griot | +20 jetons pour chaque figure qui marque |
| `harmattan` | Harmattan | ×2 mult si la main jouée compte 3 cartes ou moins |
