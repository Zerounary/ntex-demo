let data = await fs.readJson('./public/data.json')

console.log(data.length)
let items = []
for (let item of data) {
    for (let e of item.items) {
        items.push(e)
    }
}

let ids = items.map(e => {
    return e.id
})

let missId = []
let his = []
let dup = []

for (let id = 1; id <= 189; id++) {
    if(!ids.includes(id)){
        missId.push(id)
    } else {
        if(!his.includes(id)){
            his.push(id)
        } else {
            dup.push(id)
        }
    }
}

console.log(missId)
console.log(dup)

items = items.sort((a,b) => a.id - b.id)

await fs.writeJson('./public/lx.json', items)

for (let item of data) {
    let ids = []
    for (let e of item.items) {
        ids.push(e.id)
    }
    item.items = ids
}

await fs.writeJson('./public/group.json', data)