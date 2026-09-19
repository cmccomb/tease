"use strict";
(() => {
    const colors = ["#1864ab", "#c05621", "#2b8a3e", "#7048a5", "#a61e4d", "#087f8c"];
    function node(tag, text, parent) {
        const element = document.createElement(tag);
        if (text !== undefined) element.textContent = text;
        if (parent) parent.append(element);
        return element;
    }
    function svg(tag, attrs, parent, text) {
        const element = document.createElementNS("http://www.w3.org/2000/svg", tag);
        Object.entries(attrs).forEach(([key, value]) => element.setAttribute(key, String(value)));
        if (text !== undefined) element.textContent = text;
        parent.append(element);
        return element;
    }
    function plot(parent, data) {
        const chart = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        chart.setAttribute("viewBox", "0 0 640 330"); chart.setAttribute("role", "img");
        chart.setAttribute("aria-label", `${data.y_label} against ${data.x_label}`); parent.append(chart);
        const points = data.series.flatMap(series => series.points);
        const xs = points.map(p => p[0]), ys = points.map(p => p[1]);
        let xmin = Math.min(...xs), xmax = Math.max(...xs), ymin = Math.min(...ys), ymax = Math.max(...ys);
        if (data.kind === "bar") { ymin = Math.min(0, ymin); ymax = Math.max(0, ymax); }
        const dx = (xmax - xmin) || Math.max(Math.abs(xmin) * 0.1, 1);
        const dy = (ymax - ymin) || Math.max(Math.abs(ymin) * 0.1, 1);
        xmin -= dx * 0.06; xmax += dx * 0.06; ymin -= dy * 0.06; ymax += dy * 0.06;
        const x = n => 100 + (n - xmin) / (xmax - xmin) * 510;
        const y = n => 275 - (n - ymin) / (ymax - ymin) * 250;
        for (let i = 0; i <= 4; i++) {
            const a = xmin + (xmax - xmin) * i / 4, b = ymin + (ymax - ymin) * i / 4;
            svg("line", {x1:100, x2:610, y1:y(b), y2:y(b), stroke:"#dbe2e8"}, chart);
            svg("text", {x:x(a), y:295, "text-anchor":"middle", "font-size":20}, chart, Number(a.toPrecision(4)).toString());
            svg("text", {x:92, y:y(b)+4, "text-anchor":"end", "font-size":20}, chart, Number(b.toPrecision(4)).toString());
        }
        svg("text", {x:340, y:322, "text-anchor":"middle", "font-size":24}, chart, data.x_label);
        svg("text", {transform:"translate(16 150) rotate(-90)", "text-anchor":"middle", "font-size":24}, chart, data.y_label);
        const distinct = Math.max(1, new Set(xs).size);
        data.series.forEach((series, si) => {
            const color = colors[si % colors.length];
            if (data.kind === "line") svg("polyline", {points:series.points.map(p => `${x(p[0])},${y(p[1])}`).join(" "), fill:"none", stroke:color, "stroke-width":2}, chart);
            series.points.forEach(p => {
                const label = `${series.name}: ${data.x_label} ${p[0]}, ${data.y_label} ${p[1]}`;
                let mark;
                if (data.kind === "bar") {
                    const width = Math.min(40, 440 / distinct / data.series.length);
                    mark = svg("rect", {x:x(p[0]) + (si - data.series.length / 2) * width, y:Math.min(y(0),y(p[1])), width:width * 0.85, height:Math.max(1,Math.abs(y(0)-y(p[1]))), fill:color}, chart);
                } else mark = svg("circle", {cx:x(p[0]), cy:y(p[1]), r:3.5, fill:color}, chart);
                mark.setAttribute("tabindex", "0"); mark.setAttribute("aria-label", label);
                svg("title", {}, mark, label);
            });
            const legend = node("span", series.name, parent); legend.className = "legend"; legend.style.color = color;
        });
        const details = node("details", undefined, parent); node("summary", "View plot data", details);
        const table = node("table", undefined, details);
        const header = node("tr", undefined, table); ["Series",data.x_label,data.y_label].forEach(v => node("th",v,header));
        data.series.forEach(s => s.points.forEach(p => { const row = node("tr", undefined, table); [s.name,...p].forEach(v => node("td", String(v), row)); }));
    }
    function scores(parent, data) {
        const max = data.probabilities ? 1 : Math.max(...data.scores.map(s => s.score), 1e-100);
        [...data.scores].sort((a,b) => b.score-a.score).forEach(score => {
            const row = node("div", undefined, parent); row.className = "score-row";
            node("span", score.label, row);
            const bar = node("meter", undefined, row); bar.min = 0; bar.max = max; bar.value = score.score; bar.setAttribute("aria-label", score.label);
            node("span", data.probabilities ? `${(100*score.score).toFixed(1)}%` : String(score.score), row);
        });
        node("p", data.probabilities ? "Model-supplied probabilities." : "Model scores; not probabilities.", parent).className = "help";
    }
    function table(parent, data) {
        const wrapper = node("div", undefined, parent); wrapper.className = "table-scroll";
        const grid = node("table", undefined, wrapper);
        const head = node("tr", undefined, node("thead", undefined, grid));
        data.table.columns.forEach(label => node("th", label, head).setAttribute("scope", "col"));
        const body = node("tbody", undefined, grid);
        // A bounded preview; export always contains every row.
        data.table.rows.slice(0,100).forEach(row => { const tr = node("tr", undefined, body); row.forEach(cell => node("td", cell, tr)); });
        node("p", `${data.table.rows.length} rows${data.table.rows.length > 100 ? " (showing first 100)" : ""}`, parent).className = "help";
        if (data.csv !== undefined) {
            const button = node("button", "Export CSV", parent); button.type = "button";
            button.addEventListener("click", () => {
                window.tease.exportCsv(data.csv);
            });
        }
    }
    function render(parent, value) {
        parent.replaceChildren();
        if (typeof value === "string") parent.textContent = value;
        else if (value.kind === "plot") plot(parent, value.plot);
        else if (value.kind === "scores") scores(parent, value);
        else if (value.kind === "image") {
            const img = node("img", undefined, parent); img.className = "result-image";
            img.alt = "Model image"; img.src = `data:${value.image.mime};base64,${value.image.base64}`;
        }
        else if (value.kind === "table") table(parent, value);
    }
    window.teaseView = { render, node };
})();
