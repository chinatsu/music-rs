// ==UserScript==
// @name        Export album rateyourmusic.com
// @namespace   Violentmonkey Scripts
// @match       https://rateyourmusic.com/release/*/*/*/*
// @version     1.0
// @author      -
// @description 11/16/2025, 2:39:55 PM
// @grant GM_xmlhttpRequest
// ==/UserScript==

const SERVER_URL = "http://localhost:5000/update";
// const TOKEN = "your_token_here";

function getInt(val) {
  multiplier = val.substr(-1).toLowerCase();
  if (multiplier == "k") return parseInt(val) * 1000;
  else if (multiplier == "m") return parseInt(val) * 1000000;
  else return parseInt(val);
}

function getFloat(val) {
  multiplier = val.substr(-1).toLowerCase();
  if (multiplier == "k") return parseFloat(val) * 1000;
  else if (multiplier == "m") return parseFloat(val) * 1000000;
  else return parseFloat(val);
}

function getArtistName(artist) {
  return artist.firstChild.textContent.trim();
}

function getAlbumName(album) {
  return album.firstChild.textContent.trim();
}

function getArtistFromUnlinked(artist_string) {
  if (!artist_string.startsWith("[") && artist_string.endsWith("]")) {
    // we might have a localized name here
    return artist_string.split("[")[0].trim();
  }
  return artist_string;
}

// note: vibe coded
function cleanTrackTitle(uncleanTitle) {
  const title = uncleanTitle.split("\n")[0].trim();
  // Keywords that indicate the parenthetical content should be kept
  const keepKeywords =
    /\b(feat\.?|ft\.?|featuring|with|vs\.?|versus|remix|edit|mix|version|live|demo|instrumental|acoustic|reprise|part|pt\.?)\b/i;

  // Check if there's parenthetical content
  const parenMatch = title.match(/^(.+?)\s*\((.+?)\)\s*$/);
  if (!parenMatch) return title;

  const mainTitle = parenMatch[1].trim();
  const parenContent = parenMatch[2].trim();

  // Keep if parentheses contain music-related keywords
  if (keepKeywords.test(parenContent)) {
    return title;
  }

  // Check if main title has non-Latin chars and paren content is mostly Latin
  // This suggests a translation scenario
  const hasNonLatin = /[^\x00-\x7F]/.test(mainTitle);
  const parenIsLatin = /^[\x00-\x7F\s]+$/.test(parenContent);

  if (hasNonLatin && parenIsLatin) {
    // Likely a translation, remove it
    return mainTitle;
  }

  // Default: keep the full title to be safe
  return title;
}

function getTrackFromUnlinked(track_number, track) {
  const split = track.split(" - ");
  if (split.length == 2) {
    // there is likely an artist and a title here.
    var artist = getArtistFromUnlinked(split[0]);
    var title = cleanTrackTitle(split[1]);
    return { track_number, artist, title };
  }
  return { track_number, title: cleanTrackTitle(track) };
}

function copyAction(event) {
  var info = document.querySelector("table.album_info");
  var media = document.getElementById("media_link_button_container_top");
  var links = JSON.parse(media.dataset.links);
  var album = getAlbumName(document.getElementsByClassName("album_title")[0]);
  var rating = document.querySelector('meta[itemprop="ratingValue"]');
  var score = getFloat(rating ? rating.content : "0.0");
  var votes = document.querySelector('meta[itemprop="ratingCount"]');
  var voters = getInt(votes ? votes.content : "0");
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
      var songtitle = cleanTrackTitle(title.textContent.replace(" - ", ""));
      return {
        track_number: idx + 1,
        artist: getArtistFromUnlinked(artist.textContent),
        title: songtitle,
      };
    }
    if (title) {
      return {
        track_number: idx + 1,
        title: cleanTrackTitle(title.textContent),
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
    album,
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
  sendData([post]);
}

function sendData(release) {
  var headers = {
    "Content-Type": "application/json",
  };
  if (typeof TOKEN !== "undefined") {
    headers["Authorization"] = `Bearer ${TOKEN}`;
  }
  var method = {
    method: "POST",
    url: SERVER_URL,
    data: JSON.stringify(release),
    headers: headers,
    responseType: "json",
    onload: function (response) {
      console.log("sent some data :)");
      console.log(response);
    },
  };
  console.log(release);
  GM_xmlhttpRequest(method);
}

function addButtons() {
  var buttonNode = document.createElement("div");
  buttonNode.innerHTML =
    '<button id="copyButton" type="button">Copy album</button>';
  document.getElementsByClassName("album_title")[0].appendChild(buttonNode);

  document
    .getElementById("copyButton")
    .addEventListener("click", copyAction, false);
}

addButtons();
