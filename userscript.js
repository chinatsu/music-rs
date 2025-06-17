// ==UserScript==
// @name        Weekly charts rateyourmusic.com
// @namespace   Violentmonkey Scripts
// @match       https://rateyourmusic.com/charts/daily/top/album/*/*
// @grant       none
// @version     1.0
// @author      -
// @description 6/16/2025, 7:54:14 AM
// @grant GM_xmlhttpRequest
// ==/UserScript==

var SERVER_URL = "http://localhost:5000/update"


function getArtistName(artist) {
    var ui_name = artist.getElementsByClassName("ui_name_locale");
    var ui_name_original = artist.getElementsByClassName("ui_name_locale_original");
  
    if (ui_name_original.length > 0) {
      return ui_name_original[0].textContent.trim();
    }
    return ui_name[0].textContent.trim();
  }
  
  function copyAction(event) {
      var releases = [];
      for (el of document.getElementsByClassName("page_charts_section_charts_item object_release")) {
        var media = el.getElementsByClassName("media_link_container")[0];
        var links = JSON.parse(media.dataset.links);
        var album = el.getElementsByClassName("release")[0].textContent.trim();
        var artists = [].slice.call(el.getElementsByClassName("artist")).map(artist => getArtistName(artist));
        var genres = [].slice.call(el.getElementsByClassName("genre")).map(genre => genre.textContent.trim().toLowerCase());
        var date = new Date(el.getElementsByClassName("page_charts_section_charts_item_date")[0].firstElementChild.textContent.trim()).toLocaleString('sv').split(" ")[0];
        if (date == "Invalid") {
          continue
        }
        if (links && links["bandcamp"]) {
          var post = {
            artists,
            album,
            date,
            genres,
            url: Object.values(links["bandcamp"])[0]["url"]
          }
          releases.push(post);
  
        }
      }
      sendData(releases)
  }
  
  function sendData(releases) {
    console.log(releases.length);
    var method = {
      method: "POST",
      url: SERVER_URL,
      data: JSON.stringify(releases),
      headers: {
        "Content-Type": "application/json"
      },
      responseType: "json",
      onload: function(response) {
        console.log("sent some data :)")
        console.log(response)
      }
    }
    console.log(method)
    GM_xmlhttpRequest(method);
  }
  
  function parseDates() {
    var date_url = location.href.split("/").filter(n => n).at(6)
    var dates = date_url.split("-")
    var start = new Date(dates[0])
    var end = new Date(dates[1])
    return {start, end}
  }
  
  function formatDates(dates) {
    var start = dates.start.toISOString().split('T')[0].replaceAll('-', '.')
    var end = dates.end.toISOString().split('T')[0].replaceAll('-', '.')
    return `${start}-${end}`
  }
  
  function nextWeekAction(event) {
    var dates = parseDates()
    dates.start.setDate(dates.start.getDate() + 7);
    dates.end.setDate(dates.end.getDate() + 7);
    window.location.assign(`../${formatDates(dates)}`);
  }
  
  function prevWeekAction(event) {
    var dates = parseDates()
    dates.start.setDate(dates.start.getDate() - 7);
    dates.end.setDate(dates.end.getDate() - 7);
    window.location.assign(location.href.replace(/\d{4}\.\d{2}\.\d{2}-\d{4}\.\d{2}\.\d{2}/, formatDates(dates)));
  }
  
  function addButtons() {
    var match = location.href.split("/").filter(n => n).at(6).match(/\d{4}\.\d{2}\.\d{2}-\d{4}\.\d{2}\.\d{2}/);
    if (!match) {
      return;
    }
    var buttonNode = document.createElement('div');
    buttonNode.innerHTML = '<button id="prevWeek" type="button">Previous week</button><button id="copyButton" type="button">Copy albums</button><button id="nextWeek" type="button">Next week</button>';
    document.getElementById("page_charts_section_header").appendChild(buttonNode);
  
    document.getElementById("copyButton").addEventListener (
        "click", copyAction, false
    );
  
    document.getElementById("prevWeek").addEventListener (
        "click", prevWeekAction, false
    );
    document.getElementById("nextWeek").addEventListener (
        "click", nextWeekAction, false
    );
  }
  
  addButtons()
  
  