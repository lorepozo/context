const ADD = new Symbol('add')
const CONNECT = new Symbol('connect')
const EXPLORE = new Symbol('explore')

class Item { // outside use should only access .id and .data
  constructor(kn, id, data) {
    this.kn = kn
    this.id = id
    this._data = data
    this.count = [] // time-series of accesses
  }
  get data() {
    this.count.push({ time: new Date() })
    return this._data
  }
}

class Action {
  constructor(type, name, obj) {
    this.type = type // one of ADD, CONNECT, EXPLORE
    this.name = name // of mechanism
    this.obj = obj // type-dependent
  }
  perform(kn) {
    // TODO
  }
}

class Context {
  constructor(counter, kn, items, name, callback) {
    this.items = items//new Set([...itemIds].map(id => new Item(kn, id)))
    this.actions = []
    this.name = name
    counter.on('done', () => {
      // TODO (maybe): merge actions
      this.actions.forEach(action => action.perform(kn))
      callback()
    })
  }
  get() {
    return this.items
  }
  add(data) {
    this.actions.push(new Action(ADD, this.name, {data}))
  }
  connect(id1, id2) {
    this.actions.push(new Action(CONNECT, this.name, {ids: new Set([id1, id2])}))
  }
  explore(id) {
    this.actions.push(new Action(EXPLORE, this.name, {id}))
  }
}

class Network {
  constructor({contextSize, maxSize}) {
    this.graph = new Map() // id -> [adjacents]
    this.vertices = new Map() // id -> Item
    this.nextId = 1;
    this.contextSize = contextSize
    this.maxSize = maxSize
  }
  // TODO TODO TODO I left off here
}
