const numSets = 10 // loop over 10 datasets

// TODO: create data files dataN.js

function senseOnIteration(iteration) {
  iteration %= numSets
  return require(`./data${iteration}`)
}

module.exports = senseOnIteration
