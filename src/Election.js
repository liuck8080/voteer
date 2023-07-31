import React, {useState}  from 'react'
import Async from 'react-async';
import {v4 as uuidv4} from 'uuid'
 
export default function Election({name, id}) {
  let voteFor = ''

  function onChangeValue(event) {
    voteFor = event.target.value
  }

  const loadElection = async ({ election_id }) => window.contract.get_candidates({election_id:election_id });

  const ElectionHolder = () => null;
  const ElectionDetails = ({options }) => {
    const [data, setData]= useState(options);
    function reload() {
      window.contract.get_candidates({election_id:id }).then(data=>setData(data));
    }

  function handleVote() {
    if (voteFor == null || voteFor.length == 0) return; 
    let params = {
      election_id:id,
      options: [voteFor],
    }
    contract.vote(params).then(v => reload())
  }

  function handleRevoke() {
    if (voteFor == null || voteFor.length == 0) return; 
    let params = {
      election_id:id,
      options: [voteFor],
    }
    contract.revoke(params).then(v => reload())
  }
    return (
    <div>
    <table widht='100%' onChange={onChangeValue}>
    {data.map(candidate=> (
      <tbody key={uuidv4()}>
      <tr>
        <td><input type="radio" name={"election_" + id} value={candidate.name}></input></td>
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
