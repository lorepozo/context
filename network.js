const {flip} = require('./util')
const ORIENT = new Symbol('orient')
const ADD = new Symbol('add')

// Item stores .id and .data
class Item {
  constructor(kn, mechName, id, data) {
    this.kn = kn
    this.mechName = mechName
    this.id = id
    this._data = data
    this.history = [] // time-series of data accesses
  }
  get data() {
    this.history.push({ epoch: this.kn.epoch, time: new Date() })
    return this._data
  }
  count(epoch) {
    return this.history.filter(info => info.epoch == epoch).length
  }
  countSince(epoch) { // inclusive
    return this.history.filter(info => info.epoch >= epoch).length
  }
}

class Action {
  constructor(type, name, obj) {
    this.type = type // one of ORIENT, ADD
    this.name = name // of mechanism
    this.obj = obj   // {id}         if ORIENT
                     // {data, ctx}  if ADD
  }
  perform(kn) {
    switch this.type {
    case ORIENT:
      kn.orient(this.obj.id)
    case ADD:
      kn.add(this.obj.ctx, this.name, this.obj.data)
    }
  }
}

class Context {
  constructor(counter, kn, items, frontier, epoch, callback) {
    this.items = items//new Set([...itemIds].map(id => new Item(kn, id)))
    this.fronter = frontier
    this.epoch = epoch
    this.actions = []
    counter.on('done', () => {
      // TODO merge ORIENT actions (currently just using last one)
      this.actions.forEach(action => action.perform(kn))
      callback()
    })
  }
  for(name) {
    // TODO (maybe): return a thread-safe view of the context
    this.name = name
  }
  get() {
    return this.items
  }
  explore() {
    return this.frontier
  }
  orient(id) {
    this.actions.push(new Action(ORIENT, this.name, {id}))
  }
  add(data) {
    this.actions.push(new Action(ADD, this.name, {data, ctx: this}))
  }
}

class Network {
  constructor({contextSize, maxSize}) {
    this.contextSize = contextSize
    this.maxSize = maxSize
    this.graph = new Map() // id -> {adjacents}
    this.vertices = new Map() // id -> Item
    this.nextId = 1
    this.epoch = 0    // epoch counts context-switches
    this.orients = [] // of ids (.size = epoch)
  }
  orient(id) {
    assert(this.vertices.has(id))
    this.orients[this.epoch++] = id
  }
  add(ctx, mechName, data, epoch) {
    let id = this.nextId++
    this.vertices[id] = new Item(this, mechName, id, data)
    let counts = this.ctx.items.map(item => { id: item.id, count: item.countSince(ctx.epoch) })
    let sum = counts.reduce( (a, b) => a.count + b.count )
    if (sum == 0) {
      // uniform
      counts = counts.map(item => { id: item.id, count: 1 })
      sum = counts.length
    }

    this.graph[id] = new Set()
    // bernoulli of count/sum for each item in context
    for (const item of counts) {
      let p = item.count / sum
      let connect = flip(p)
      if (connect) {
        this.graph[id].add(item.id)
      }
    }
    // still no connections? assign each item to a region in the
    // interval [0,1] of size count/sum, then use random() to pick.
    // this is equivalent to one time-step of a CRP with no new tables.
    if (this.graph[id].size == 0) {
      let r = Math.random()
      for (const item of counts) {
        r -= item.count / sum
        if (r < 0) {
          this.graph[id].add(item.id)
          break
        }
      }
    }
  }
  newContext(counter, callback) {
    // TODO construct items, frontier
    return new Context(counter, this, items, frontier, this.epoch, callback)
  }
}

module.Exports = { Network }
