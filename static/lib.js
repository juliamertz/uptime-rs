/**
 * @param {string} selector
 * @returns {Element}
 */
function $(selector) {
  return document.querySelector(selector);
}

/**
 * @param {Element} element
 * @param {string} name
 * @returns {string}
 */
function attr(element, name) {
  return element.getAttribute(name);
}

/**
 * Extract attribute key-value pairs from an element
 * @param {Element} element
 * @param {string[]} attributes - Array of attribute names
 * @returns {Record<string, string>} - Object with attribute key-value pairs
 */
function attributes(element, attribute_names) {
  const result = {};
  for (let key of attribute_names) {
    const value = element.getAttribute(key);
    if (key.startsWith("data-")) key = key.slice(5);

    result[key] = value;
  }

  return result;
}
