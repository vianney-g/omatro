.pragma library

// Displayed text: hand names, ranks, suits, effects.

var HANDS = {
  high_card: "High card",
  pair: "Pair",
  two_pair: "Two pair",
  three_of_a_kind: "Three of a kind",
  straight: "Straight",
  flush: "Flush",
  full_house: "Full house",
  four_of_a_kind: "Four of a kind",
  straight_flush: "Straight flush"
}

var SUITS = { hearts: "♥", diamonds: "♦", clubs: "♣", spades: "♠" }

function hand(kind) { return HANDS[kind] || "" }

function rank(r) { return r <= 10 ? String(r) : ["J", "Q", "K", "A"][r - 11] }

function suit(s) { return SUITS[s] || "" }

function isRed(s) { return s === "hearts" || s === "diamonds" }

// English number format: 12,345 and 2.5.
function number(n) {
  return String(Math.floor(n)).replace(/\B(?=(\d{3})+(?!\d))/g, ",")
}

function money(n) { return "$" + n }

function mult(m) {
  return Number.isInteger(m) ? String(m) : m.toFixed(1)
}

// An engine effect: {"chips": 11}, {"mult": 2} or {"x_mult": 2}.
function effect(e) {
  if (e.chips !== undefined) return "+" + e.chips
  if (e.mult !== undefined) return "+" + mult(e.mult) + " mult"
  if (e.x_mult !== undefined) return "×" + mult(e.x_mult) + " mult"
  return ""
}

function isChips(e) { return e.chips !== undefined }
