function workflow() {
   const repo = 'Walkmana-25/Sapphillon';
   const user = 'Walkmana-25';
   const github_search_url = `https://github.com/search?q=author%3A${user}+repo%3A${repo}&type=commits&s=committer-date&o=desc`;
   const tabId = JSON.parse(floorpCreateTab(github_search_url, false)).id;
   let commits_html = floorpTabElement(tabId, 'div.Box-sc-62in7e-0.gwXVXe', 5000);
   floorpNavigateTab(tabId, 'https://myactivity.google.com/product/maps');
   floorpTabWaitForElement(tabId, 'span.hFYxqd', 5000);
   let location_html = floorpTabElement(tabId, 'div.vwWeec', 1000000);
   console.log(location_html);
   let url = 'http://host.docker.internal:5050/generate?q=' +  encodeURIComponent(`${commits_html}\n\n${location_html}\n\n上記のGitHubのコミット履歴とGoogleマップの行動履歴から、その人の作業日報を作成してください。`);
   let llm_res = fetch(url);
   writeFile('./example/nippou_result.txt', llm_res);
   console.log(llm_res);
}

workflow();

function get_github_commits(repo, user) {
   const github_search_url = `https://github.com/search?q=author%3A${user}+repo%3A${repo}&type=commits&s=committer-date&o=desc`;
   const tabId = JSON.parse(floorpCreateTab(github_search_url, false)).id;
   let commits_html = floorpTabElement(tabId, 'div.Box-sc-62in7e-0.gwXVXe', 5000);
   return commits_html;
}

function get_google_maps_location() {
   const tabId = JSON.parse(floorpCreateTab('https://myactivity.google.com/product/maps', false)).id;
   floorpTabWaitForElement(tabId, 'span.hFYxqd', 5000);
   let location_html = floorpTabElement(tabId, 'div.vwWeec', 1000000);
   return location_html;
}

function generate_nippou() {
   const repo = 'Walkmana-25/Sapphillon';
   const user = 'Walkmana-25';
   let commits_html = get_github_commits(repo, user);
   let location_html = get_google_maps_location();
   let url = 'http://host.docker.internal:5050/generate?q=' +  encodeURIComponent(`${commits_html}\n\n${location_html}\n\n上記のGitHubのコミット履歴とGoogleマップの行動履歴から、その人の作業日報を作成してください。`);
   let llm_res = fetch(url);
   writeFile('./example/nippou_result.txt', llm_res);
   console.log(llm_res);
}

generate_nippou();