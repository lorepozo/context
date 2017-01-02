
/************************/
/***** string-based *****/
/************************/

function fromItems(items) {
  const combinators = new Set()
  for (const item of items) {
    if (item.mechName === this.name) {
      for (const combinator of item.data) {
        combinators.add(combinator)
      }
    }
  }
  return combinators
}

function fromContext(ctx) {
  const getCombs = fromItems(ctx.get())
  const exploreCombs = fromItems(ctx.explore())
  return {getCombs, exploreCombs}
}

function mostProbable(probCombs) {
  return nMostProbable(probCombs, 1)[0]
}

function nMostProbable(probCombs, n) {
  const mostProbable =
    [...probCombs].sort((a, b) => a[1]-b[1]).slice(0, n).map(cp => cp[0])
  // add 'undefined' until appropriate length,
  // replace all 'undefined' with 'null'
  mostProbable.length = n
  return mostProbable.map(x => (x === undefined) ? null : x)
}

function findContainer(comb, items) {
  for (const item of items) {
    if (item.mechName === this.name) {
      for (const combinator of item.data) {
        if (combinator === comb) {
          return item
        }
      }
    }
  }
  return null
}

/************************/
/***** class-based ******/
/************************/

class Comb {
  // TODO
}

module.exports = { fromContext, mostProbable, nMostProbable, findContainer }
