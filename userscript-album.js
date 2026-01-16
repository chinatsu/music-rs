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
        .map((mood) => mood.trim().toLowerCase())
    )
    .filter((mood) => mood);
  var tracks = [
    ...document.querySelectorAll(
      "#tracks > .track > .tracklist_line > .tracklist_title"
    ),
  ].map((song, idx) => {
    var artist = song.querySelector(".artist");
    var title = song.querySelector(".song");
    if (artist) {
      var songtitle = title.textContent.replace(" - ", "");
      return {
        track_number: idx + 1,
        artist: artist.textContent,
        title: songtitle,
      };
    }
    return { track_number: idx + 1, title: title.textContent };
  });
  var dateString = [...info.querySelectorAll("th.info_hdr")].filter((element) =>
    element.textContent.match("Released")
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
  var method = {
    method: "POST",
    url: SERVER_URL,
    data: JSON.stringify(release),
    headers: {
      "Content-Type": "application/json",
      // "Authorization": "Bearer " + TOKEN
    },
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
