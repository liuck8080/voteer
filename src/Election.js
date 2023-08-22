import React, {useState, useRef}  from 'react'
import Async from 'react-async';
import {v4 as uuidv4} from 'uuid'
 
export default function Election({name, id, mutiple}) {
  const checkedOptions = useRef([]);

  function onChangeValue(e) {
    const { value, checked } = e.target;
    if (checked) {
      if (!checkedOptions.current.includes(value)) {if(mutiple) {
        checkedOptions.current.push(value)
        } else {
          checkedOptions.current[0] = value;
        };
      }
    } else {
      checkedOptions.current=checkedOptions.current.filter((e) => e !== value);
    }
  }

  const loadElection = async ({ election_id }) => window.contract.get_candidates({election_id:election_id });

  const ElectionHolder = () => null;
  const ElectionDetails = ({options }) => {
    const [data, setData]= useState(options);
    function reload() {
      checkedOptions = [];
      window.contract.get_candidates({election_id:id }).then(data=>setData(data));
    }

  function handleVote() {
    if (checkedOptions.current == null || checkedOptions.current.length == 0) return;
    let params = {
      election_id:id,
      options: checkedOptions.current,
    }
    contract.vote(params).then(v => reload())
  }

  function handleRevoke() {
    if (checkedOptions.current == null || checkedOptions.current.length == 0) return;
    let params = {
      election_id:id,
      options: checkedOptions.current,
    }
    contract.revoke(params).then(v => reload())
  }
    return (
    <div>
    <table widht='100%' >
    {data.map((candidate, idx)=> (
      <tbody key={uuidv4()}>
      <tr>
        <td><input type={mutiple ? "checkbox": "radio"} name={"election_" + id} value={candidate.name} onChange={onChangeValue}></input></td>
        <td>{candidate.name}</td><td></td>
      </tr>
      <tr>
        <td>&nbsp;</td>
        <td>&nbsp;</td>
        <td>{candidate.supported}</td>
      </tr>
      </tbody>
    ))}
    </table>
    <button onClick={handleVote}>vote</button>&nbsp;&nbsp;&nbsp;&nbsp;
    <button onClick={handleRevoke}>revoke</button>
    </div>
  );}


  return (
    <>
    <h2>{name}</h2>
    <Async promiseFn={loadElection} election_id={id}>
    <Async.Pending>
      <ElectionHolder />
    </Async.Pending>
    <Async.Fulfilled>{data => <ElectionDetails options={data} />}</Async.Fulfilled>
    <Async.Rejected>{error => <p>{error.message}</p>}</Async.Rejected>
  </Async>
  </>
  )
}
