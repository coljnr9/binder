const chromium = require("@sparticuz/chromium")
const puppeteer = require("puppeteer-core")
var { Readability } = require('@mozilla/readability');
const { JSDOM } = require('jsdom');

function delay(time) {
    return new Promise(function(resolve) {
        setTimeout(resolve, time)
    })
}

exports.handler = async (event, context) => {
    console.log("In handler - event: ", event);

    const articleUrl = event.articleUrl; 
    console.log("Processing url: ", articleUrl);

    const browser = await puppeteer.launch({
        args: chromium.args,
        defaultViewport: chromium.defaultViewport,
        executablePath: await chromium.executablePath(),
        ignoreHTTPSErrors: true,
        headless: 'new',
    });
    console.log("Created browser");

    const page = await browser.newPage();
    console.log("Created new page");

    await page.goto(articleUrl);
    let content = await page.content();
    console.log("Got page content: ", content);

    const dom = new JSDOM(content);
    let reader = new Readability(dom.window.document);
    let parsed = reader.parse();

    await page.close();
    await browser.close();

    return parsed
}
