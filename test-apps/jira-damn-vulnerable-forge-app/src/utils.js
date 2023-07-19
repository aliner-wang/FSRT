import api, { route } from '@forge/api';
import {testFunctionFromTestFile} from './testfile';

export async function fetchIssueSummary(issueIdOrKey, url) {

  let obj = {
    method: 'PTACH',
    bananas: 'apple',
    headers: { //
      Accept: 'application/json',
    },
  };

  let val = "grapefruit";

  val = "peach";

  let pre_url = "/rest/api/3/issue/" + val;

  let a_url = route`/rest/api/3/issue/${issueIdOrKey}?fields=summary/${val}`;

  const resp = await api
    .asApp()
    .requestJira(get_route(), obj);
  const data = await resp.json();
  console.log(JSON.stringify(data));
  return data['fields']['summary'];
}

function get_route() {
  //return a_url = route`/rest/api/3/issue/${issueIdOrKey}?fields=summary`;
  return route`/bananas/${issueIdOrKey}?fields=summary`;
}

export async function writeComment(issueIdOrKey, comment) {
  /* const api = require('@forge/api'); */


  testFunctionFromTestFile("test_value_param")

  const resp = await api
    .asApp()
    .requestJira(route`/rest/api/3/issue/${issueIdOrKey}/comment`, {
      method: 'POST',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json',
      },
      body: `{
        "body": {
          "type": "doc",
          "version": 1,
          "content": [
            {
              "type": "paragraph",
              "content": [
                "text": ${comment},
                "type": "text"
              ]
            }
          ]
        }
      }`,
    });
  console.log(`Response: ${resp.status} ${resp.statusText}`);
  console.log(await resp.json());
}
