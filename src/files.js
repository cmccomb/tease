"use strict";
(() => {
    function parseCSV(text) {
        text = text.replace(/^\uFEFF/, "");
        const rows = []; let row = [], cell = "", quoted = false, closed = false;
        function field() { row.push(cell); cell = ""; closed = false; }
        function line() { field(); if (row.length > 1 || row[0] !== "") rows.push(row); row = []; }
        for (let i = 0; i < text.length; i++) {
            const c = text[i];
            if (quoted) {
                if (c === '"' && text[i+1] === '"') { cell += '"'; i++; }
                else if (c === '"') { quoted = false; closed = true; }
                else cell += c;
            } else if (c === ',' ) field();
            else if (c === '\n' || c === '\r') { line(); if (c === '\r' && text[i+1] === '\n') i++; }
            else if (c === '"' && !cell && !closed) quoted = true;
            else { if (closed || c === '"') throw new Error('Invalid CSV quoting.'); cell += c; }
            if (rows.length > 1001 || row.length > 64 || cell.length > 8192) throw new Error('CSV exceeds its row, column, or cell limits.');
        }
        if (quoted) throw new Error('CSV has an unclosed quoted field.');
        if (cell || row.length || closed) line();
        const columns = rows.shift() || [];
        if (!columns.length || columns.length > 64 || columns.some(c => !c.trim()) || new Set(columns).size !== columns.length || rows.some(r => r.length !== columns.length) || rows.length > 1000) throw new Error('CSV needs unique headers and rows of matching width (up to 1,000 rows).');
        return {columns, rows};
    }
    function apply(input, raw, preview, name = 'Preset file', invalidate = true) {
        if (invalidate) input._ticket = (input._ticket || 0) + 1; input._reading = false;
        input._payload = ''; input._fileName = name;
        preview.replaceChildren();
        if (!raw) return;
        let data;
        if (input.dataset.fileKind === 'image') {
            const image = JSON.parse(raw);
            if (!['image/png','image/jpeg'].includes(image.mime)) throw new Error('Select a PNG or JPEG image.');
            data = {kind:'image',image};
        } else { data = {kind:'table',table:parseCSV(raw)}; }
        window.teaseView.render(preview, data);
        window.teaseView.node('p', name, preview).className = 'help';
        input._payload = raw;
    }
    function read(file, kind) {
        if (!file) return Promise.resolve('');
        if (file.size > (kind === 'image' ? 8 : 2) * 1024 * 1024) return Promise.reject(new Error('File exceeds the size limit.'));
        return new Promise((resolve,reject) => {
            const reader = new FileReader();
            reader.onerror = () => reject(new Error('Could not read the selected file.'));
            reader.onload = () => {
                try {
                    if (kind === 'table') resolve(String(reader.result));
                    else {
                        const base64 = String(reader.result).split(',')[1] || '';
                        const mime = base64.startsWith('iVBORw0KGgo') ? 'image/png' : base64.startsWith('/9j/') ? 'image/jpeg' : '';
                        if (!mime) throw new Error('Select a PNG or JPEG image.');
                        resolve(JSON.stringify({mime,base64}));
                    }
                } catch (error) { reject(error); }
            };
            if (kind === 'image') reader.readAsDataURL(file); else reader.readAsText(file,'UTF-8');
        });
    }
    window.teaseFiles = {apply,read,parseCSV};
})();
