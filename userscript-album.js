// ==UserScript==
// @name        Export album rateyourmusic.com
// @namespace   Violentmonkey Scripts
// @match       https://rateyourmusic.com/release/*/*/*
// @downloadURL https://github.com/chinatsu/music-rs/blob/main/userscript-album.js
// @grant GM_xmlhttpRequest
// @grant GM_setValue
// @grant GM_getValue
// @version     1.0
// @author      -
// @description 11/16/2025, 2:39:55 PM
// @grant GM_xmlhttpRequest
// ==/UserScript==

function getServerUrl() {
  return GM_getValue('SERVER_URL');
}

function getToken() {
  return GM_getValue('TOKEN');
}

function setServerUrl(url) {
  GM_setValue('SERVER_URL', url);
}

function setToken(token) {
  GM_setValue('TOKEN', token);
}

function showConfigDialog() {
  const existingModal = document.getElementById('configModal');
  if (existingModal) {
    existingModal.remove();
  }

  const currentUrl = getServerUrl() ? getServerUrl() : '';
  const currentToken = getToken() ? getToken() : '';

  const modal = document.createElement('div');
  modal.id = 'configModal';
  modal.innerHTML = `
    <div style="
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background: rgba(0, 0, 0, 0.8);
      z-index: 10000;
      display: flex;
      align-items: center;
      justify-content: center;
    ">
      <div style="
        background: white;
        padding: 20px;
        border-radius: 8px;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        max-width: 500px;
        width: 90%;
      ">
        <h3>Configuration</h3>
        <div style="margin-bottom: 15px;">
          <label class="info_hdr">Server URL</label>
          <input type="text" id="serverUrlInput" value="${currentUrl}" style="
            width: 100%;
            padding: 8px;
            font-size: 14px;
          ">
        </div>

        <div style="margin-bottom: 20px;">
          <label class="info_hdr">API Token</label>
          <input id="tokenInput" value="${currentToken}" style="
            width: 100%;
            padding: 8px;
            font-size: 14px;
          ">
        </div>

        <div style="text-align: right;">
          <button id="cancelConfig" class="btn">Cancel</button>
          <button id="saveConfig" class="btn blue_btn">Save</button>
        </div>
      </div>
    </div>
  `;

  document.body.appendChild(modal);

  document.getElementById('cancelConfig').addEventListener('click', () => {
    modal.remove();
  });

  document.getElementById('saveConfig').addEventListener('click', () => {
    const newUrl = document.getElementById('serverUrlInput').value.trim();
    const newToken = document.getElementById('tokenInput').value.trim();
    setServerUrl(newUrl);
    setToken(newToken);

    modal.remove();
    displayAlbumInfo();
  });

  modal.addEventListener('click', (e) => {
    if (e.target === modal) {
      modal.remove();
    }
  });

  document.getElementById('serverUrlInput').focus();
}

function getNumber(val, func) {
  multiplier = val.substr(-1).toLowerCase();
  if (multiplier == "k") return func(val) * 1000;
  else if (multiplier == "m") return func(val) * 1000000;
  else return func(val);
}

function getArtistName(artist) {
  var localized_element = artist.children[0];
  var localized_name = null;
  if (
    localized_element !== undefined &&
    localized_element.tagName === "SPAN" &&
    localized_element.textContent.startsWith("[") &&
    localized_element.textContent.endsWith("]")
  ) {
    localized_name = localized_element.textContent.trim().slice(1, -1);
  }
  return { name: artist.firstChild.textContent.trim(), localized_name };
}

function getAlbumName(album) {
  return album.firstChild.textContent.trim();
}

function getArtistFromUnlinked(artist_string) {
  if (!artist_string.startsWith("[") && artist_string.endsWith("]")) {
    // we might have a localized name here
    var artist_split = artist_string.split("[");
    return {
      name: artist_split[0].trim(),
      localized_name: artist.split[1].trim().slice(0, -1),
    };
  }
  return { name: artist_string, localized_name: null };
}

function cleanTrackTitle(uncleanTitle) {
  const title = uncleanTitle.split("\n")[0].trim();
  const keepKeywords =
    /\b(feat\.?|ft\.?|featuring|with|vs\.?|versus|remix|edit|mix|version|live|demo|instrumental|acoustic|reprise|part|pt\.?)\b/i;

  const parenMatch = title.match(/^(.+?)\s*\((.+?)\)\s*$/);
  if (!parenMatch) return [title, null];

  const mainTitle = parenMatch[1].trim();
  const parenContent = parenMatch[2].trim();

  if (keepKeywords.test(parenContent)) {
    return [title, null];
  }

  const hasNonLatin = /[^\x00-\x7F]/.test(mainTitle);
  const parenIsLatin = /^[\x00-\x7F\s]+$/.test(parenContent);

  if (hasNonLatin && parenIsLatin) {
    return [mainTitle, parenContent];
  }

  return [title, null];
}

function getTrackFromUnlinked(track_number, track) {
  const split = track.split(" - ");
  if (split.length == 2) {
    // there is likely an artist and a title here.
    var artist = getArtistFromUnlinked(split[0]);
    var [title, localized_title] = cleanTrackTitle(split[1]);
    return {
      track_number,
      artist,
      title,
      localized_title,
    };
  }
  var [title, localized_title] = cleanTrackTitle(track);
  return { track_number, title, localized_title };
}

function getInformation() {
  var info = document.querySelector("table.album_info");
  var media = document.getElementById("media_link_button_container_top");
  var links = JSON.parse(media.dataset.links);
  var album_element = document.getElementsByClassName("album_title")[0];
  var album = getAlbumName(album_element);
  var [album_title, localized_title] = cleanTrackTitle(album);
  if (album_element.children[0].tagName == "SPAN") {
    localized_title = album_element.children[0].textContent;
  }
  var rating = document.querySelector('meta[itemprop="ratingValue"]');
  var score = getNumber(rating ? rating.content : "0.0", parseFloat);
  var votes = document.querySelector('meta[itemprop="ratingCount"]');
  var voters = getNumber(votes ? votes.content : "0", parseInt);
  var artists = [].slice
    .call(info.getElementsByClassName("artist"))
    .map((artist) => getArtistName(artist));
  var genres = [].slice
    .call(info.getElementsByClassName("genre"))
    .map((genre) => genre.textContent.trim().toLowerCase());
  var moods = [].slice
    .call(
      info
        .querySelector(".release_pri_descriptors")
        .textContent.split(", ")
        .map((mood) => mood.trim().toLowerCase()),
    )
    .filter((mood) => mood);
  var tracks = [
    ...document.querySelectorAll(
      "#tracks > .track > .tracklist_line > .tracklist_title",
    ),
  ].map((song, idx) => {
    var artist = song.querySelector(".artist");
    var title = song.querySelector(".song");
    if (artist && title) {
      var [songtitle, localized_title] = cleanTrackTitle(
        title.textContent.replace(" - ", ""),
      );
      return {
        track_number: idx + 1,
        artist: getArtistFromUnlinked(artist.textContent),
        title: songtitle,
        localized_title,
      };
    }
    if (title) {
      [songtitle, localized_title] = cleanTrackTitle(title.textContent);
      return {
        track_number: idx + 1,
        title: songtitle,
        localized_title,
      };
    }
    // at this point, we know the element is plaintext
    return getTrackFromUnlinked(idx + 1, song.textContent.trim());
  });
  var dateString = [...info.querySelectorAll("th.info_hdr")].filter((element) =>
    element.textContent.match("Released"),
  )[0].nextSibling.textContent;
  if (dateString.split(" ").length != 3) {
    if (dateString.split(" ").length === 1) {
      dateString = `1 1 ${dateString}`;
    } else {
      dateString = `1 ${dateString}`;
    }
  }
  //dateString = "30 March 2014"
  var date = new Date(dateString).toLocaleString("sv").split(" ")[0];

  if (date == "Invalid") {
    console.log("sorry bud");
    return;
  }
  var post = {
    artists,
    album: album_title,
    localized_title,
    date,
    genres,
    moods,
    score,
    voters,
    tracks,
    url:
      "bandcamp" in links
        ? Object.values(links["bandcamp"])[0]["url"]
        : location.href,
    rym_url: location.href,
  };
  return post;
}

function copyAction(event) {
  const post = getInformation();
  sendData([post]);
}

function sendData(release) {
  const serverUrl = getServerUrl();
  const token = getToken();

  if (serverUrl === undefined || serverUrl.trim() === '') {
    alert("Server URL is not set, please configure");
    return;
  }

  var headers = {
    "Content-Type": "application/json",
  };
  if (token && token.trim() !== '') {
    headers["Authorization"] = `Bearer ${token}`;
  }
  var method = {
    method: "POST",
    url: serverUrl,
    data: JSON.stringify(release),
    headers: headers,
    responseType: "json",
    onload: function (response) {
      console.log(response);
    },
  };
  GM_xmlhttpRequest(method);
}

function displayAlbumInfo() {
  const info = getInformation();
  const previewDiv = document.getElementById("albumPreview");

  if (info && previewDiv) {
    const displayText = `Album: ${info.album}${info.localized_title ? ` | ${info.localized_title}` : ''}
Artists: ${info.artists.map(a => a.localized_name ? `${a.name} | ${a.localized_name}` : a.name).join(', ')}
Date: ${info.date}
Score: ${info.score} (${info.voters} voters)
Genres: ${info.genres.join(', ')}
Moods: ${info.moods.join(', ')}
Tracks: ${info.tracks.length}
${info.tracks.map(t => {
      const trackArtist = t.artist ? (t.artist.localized_name ? `${t.artist.name} | ${t.artist.localized_name}` : t.artist.name) : '';
      const trackTitle = t.localized_title ? `${t.title} | ${t.localized_title}` : t.title;
      return `  ${t.track_number}. ${trackArtist ? trackArtist + ' - ' : ''}${trackTitle}`;
    }).join('\n')}
URL: ${decodeURI(info.url)}
RYM URL: ${decodeURI(info.rym_url)}`;

    previewDiv.textContent = displayText;
  }
}

function addMetadata() {
  var buttonNode = document.createElement("div");
  buttonNode.innerHTML =
    '<div id="albumPreview" style="background: #f0f0f0; padding: 10px; margin: 10px 0; border-radius: 5px; font-family: monospace; font-size: 12px; white-space: pre-wrap; overflow-y: auto;"></div>' +
    '<button id="configButton" class="btn btn_small" type="button" style="margin-right: 5px;">Configure</button>' +
    '<button id="copyButton" class="btn blue_btn btn_small" type="button">Copy album</button>';

  document.getElementsByClassName("section_main_info")[0].prepend(buttonNode);

  // Display the album information immediately
  displayAlbumInfo();

  document
    .getElementById("configButton")
    .addEventListener("click", showConfigDialog, false);

  document
    .getElementById("copyButton")
    .addEventListener("click", copyAction, false);
}

addMetadata();