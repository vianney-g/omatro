.pragma library

// Textes affichés : noms des mains, rangs, couleurs, effets.

var HANDS = {
  high_card: "Carte haute",
  pair: "Paire",
  two_pair: "Double paire",
  three_of_a_kind: "Brelan",
  straight: "Suite",
  flush: "Couleur",
  full_house: "Full",
  four_of_a_kind: "Carré",
  straight_flush: "Quinte flush"
}

var SUITS = { hearts: "♥", diamonds: "♦", clubs: "♣", spades: "♠" }

function hand(kind) { return HANDS[kind] || "" }

function rank(r) { return r <= 10 ? String(r) : ["V", "D", "R", "A"][r - 11] }

function suit(s) { return SUITS[s] || "" }

function isRed(s) { return s === "hearts" || s === "diamonds" }

// Nombres à la française : 12 345, 2,5.
function number(n) {
  return String(Math.floor(n)).replace(/\B(?=(\d{3})+(?!\d))/g, " ")
}

function money(n) { return n + " $" }

function mult(m) {
  return Number.isInteger(m) ? String(m) : m.toFixed(1).replace(".", ",")
}

// Un effet du moteur : {"chips": 11}, {"mult": 2} ou {"x_mult": 2}.
function effect(e) {
  if (e.chips !== undefined) return "+" + e.chips
  if (e.mult !== undefined) return "+" + mult(e.mult) + " mult"
  if (e.x_mult !== undefined) return "×" + mult(e.x_mult) + " mult"
  return ""
}

function isChips(e) { return e.chips !== undefined }
