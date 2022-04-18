import React, {useRef, useState} from 'react'

export default function ElectionCreate({contract}) {
    const electionTitleRef = useRef()
    const electionCandidatesRef = useRef()
    const [multiple, setMultiple] = useState(false)
    
    function handleCreate(e) {
        let name = electionTitleRef.current.value.trim();
        if (name === '') return;
        let options = electionCandidatesRef.current.value.split(/\r?\n/).map(s=>s.trim()).filter(s=>s.length > 0)
        if (options.length == 0) return;
        let params = {
            candidates:options,
            multiple: multiple,
            name: name,
        }
        contract.create_election(params).then(v => console.log(v))
        console.log(v)
        console.log(params)
    }

    function onChangeValue(event) {
        setMultiple(event.target.value == "multiple")
    }
  return (
    <>
    <h2> create a new election</h2>
    <fieldset>
    <div>
        <label>Title: <input type="text" name="title" id ="title" ref={electionTitleRef}/></label>
    </div>
    <div>
        <label>options(each line for an option):<br/>
        <textarea name="options" rows="5" cols="120" ref={electionCandidatesRef}/></label>
    </div>
    <div onChange={onChangeValue}>
        <input type="radio" name="type" id="single" value="single" defaultChecked/>single
        <input type="radio" name="type" id="mutiple" value="multiple"/>mutiple
    </div>
        <button onClick={handleCreate}>create</button>
    </fieldset>
    </>
  )
}
