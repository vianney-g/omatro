.pragma library

// Displayed text: ranks, suits, effects, numbers (hand names come from the engine).

var SUITS = { hearts: "♥", diamonds: "♦", clubs: "♣", spades: "♠" }

function rank(r) { return r <= 10 ? String(r) : ["J", "Q", "K", "A"][r - 11] }

function suit(s) { return SUITS[s] || "" }

function isRed(s) { return s === "hearts" || s === "diamonds" }

// English number format: 12,345 and 2.5, then 1.234e11 past a hundred billion.
// Infinite targets come from the engine as null.
function number(n) {
  if (n === null || !isFinite(n)) return "\u221e"
  n = Math.floor(n)
  if (n >= 1e11) {
    var e = Math.floor(Math.log10(n))
    return (n / Math.pow(10, e)).toFixed(3) + "e" + e
  }
  return String(n).replace(/\B(?=(\d{3})+(?!\d))/g, ",")
}

function money(n) { return n < 0 ? "−$" + (-n) : "$" + n }

// "1 card", "2 cards", "1–3 cards".
function range(min, max) {
  return (min === max ? String(min) : min + "–" + max) + (max === 1 ? " card" : " cards")
}

function mult(m) {
  return Number.isInteger(m) ? String(m) : m.toFixed(1)
}

// An engine effect: {"chips": 11}, {"mult": 2}, {"x_mult": 2}, {"money": 2} or "again".
function effect(e) {
  if (e === "again") return "Again!"
  if (e === "debuffed") return "Debuffed"
  if (e === "balance") return "Balanced"
  if (e.money !== undefined) return "+" + money(e.money)
  if (e.chips !== undefined) return "+" + e.chips
  if (e.mult !== undefined) return "+" + mult(e.mult) + " mult"
  if (e.x_mult !== undefined) return "×" + mult(e.x_mult) + " mult"
  return ""
}

function isChips(e) { return e.chips !== undefined }

function isMoney(e) { return e.money !== undefined }

var BLIND_STATUSES = { upcoming: "Upcoming", select: "Next", current: "In progress", defeated: "Defeated", skipped: "Skipped" }

function blindStatus(s) { return BLIND_STATUSES[s] || "" }

// ---- glossary: terms card texts mention (from the engine's `glossary`)

var termPattern = null
var termCount = -1
var termsByName = {}

// One pattern for every term, longest first so that "Fork Bomb" wins over
// "Fork"; a name matches as a whole word, plural allowed ("Daemon cards").
function pattern(terms) {
  if (terms.length !== termCount) {
    termCount = terms.length
    termsByName = {}
    for (var i = 0; i < terms.length; i++) termsByName[terms[i].name] = terms[i]
    var names = terms.map(function(t) { return t.name })
      .sort(function(a, b) { return b.length - a.length })
      .map(function(n) { return n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") })
    termPattern = names.length
      ? new RegExp("(^|[^A-Za-z0-9])(" + names.join("|") + ")(?=s?(?![A-Za-z0-9]))", "g")
      : null
  }
  return termPattern
}

// Whether a matched name is explained (hand types only block matches), and
// not the card's own name.
function explained(name, own) {
  var t = termsByName[name]
  return t !== undefined && t.description !== "" && name !== own
}

// The terms a text mentions, in order, each once (not the card's own name).
function mentions(text, terms, own) {
  var re = pattern(terms)
  if (!re || !text) return []
  var found = [], seen = {}
  re.lastIndex = 0
  var m
  while ((m = re.exec(text)) !== null) {
    if (explained(m[2], own) && !seen[m[2]]) {
      seen[m[2]] = true
      found.push(termsByName[m[2]])
    }
  }
  return found
}

// The text as StyledText, its explained terms underlined.
function highlight(text, terms, own) {
  if (!text) return ""
  var escaped = text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
  var re = pattern(terms)
  if (!re) return escaped
  re.lastIndex = 0
  return escaped.replace(re, function(all, before, name) {
    return explained(name, own) ? before + "<u>" + name + "</u>" : all
  })
}
