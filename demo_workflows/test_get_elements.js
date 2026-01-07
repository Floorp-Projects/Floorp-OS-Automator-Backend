function workflow() {
  var tab = floorp.createTab("https://example.com", false);
  try {
    var els = floorp.tabGetElements(tab, "h1");
    console.log("Elements H1: " + JSON.stringify(els));

    var parsed = JSON.parse(els);
    if (parsed.elements && parsed.elements.length > 0) {
      var firstId = parsed.elements[0];
      console.log("First ID: " + firstId);

      // Try to use ID as selector
      try {
        var text = floorp.tabElementText(tab, firstId);
        console.log("Text by ID: " + text);
      } catch (e) {
        console.log("Failed to get text by ID: " + e);
      }
    }
  } catch (e) {
    console.log("Error: " + e);
  } finally {
    floorp.destroyTabInstance(tab);
  }
}
