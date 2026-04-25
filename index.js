// Cloudflare Worker: index.js
// Handles all DB actions for Rust client via HTTP POST
// Expects JSON body: { action, table, ... }
// Only allows specific actions (no custom SQL)
// Requires environment variables: TURSO_URL, TURSO_AUTH_TOKEN

export default {
  async fetch(request, env, ctx) {
    // Handle CORS preflight
    if (request.method === 'OPTIONS') {
      return new Response(null, {
        status: 204,
        headers: {
          'Access-Control-Allow-Origin': '*',
          'Access-Control-Allow-Methods': 'POST, OPTIONS',
          'Access-Control-Allow-Headers': 'Content-Type',
        },
      });
    }
    if (request.method !== 'POST') {
      return new Response('Method Not Allowed', {
        status: 405,
        headers: { 'Access-Control-Allow-Origin': '*' },
      });
    }
    let data;
    try {
      data = await request.json();
    } catch (e) {
      return new Response('Invalid JSON', {
        status: 400,
        headers: { 'Access-Control-Allow-Origin': '*' },
      });
    }
    const { action, table, id, record, column, value } = data;
    if (!action || !table) {
      return new Response('Missing action or table', {
        status: 400,
        headers: { 'Access-Control-Allow-Origin': '*' },
      });
    }
    // Prepare DB request
    let sql, params = [];
    switch (action) {
      case 'fetch':
        sql = `SELECT * FROM ${table}`;
        break;
      case 'fetch_by_id':
        sql = `SELECT * FROM ${table} WHERE id = ?`;
        params = [id];
        break;
      case 'insert':
        if (!record) return new Response('Missing record', {
          status: 400,
          headers: { 'Access-Control-Allow-Origin': '*' },
        });
        const keys = Object.keys(record).filter(k => k !== 'id');
        const cols = keys.join(', ');
        const placeholders = keys.map(_ => '?').join(', ');
        sql = `INSERT INTO ${table} (${cols}) VALUES (${placeholders})`;
        params = keys.map(k => wrap_param(record[k]));
        break;
      case 'update':
        if (!record || record.id == null) return new Response('Missing record or id', {
          status: 400,
          headers: { 'Access-Control-Allow-Origin': '*' },
        });
        const updateKeys = Object.keys(record).filter(k => k !== 'id');
        const setClause = updateKeys.map(k => `${k} = ?`).join(', ');
        sql = `UPDATE ${table} SET ${setClause} WHERE id = ?`;
        params = updateKeys.map(k => wrap_param(record[k]));
        params.push(wrap_param(record.id));
        break;
      case 'update_by_column':
        if (!id || !column) return new Response('Missing id or column', {
          status: 400,
          headers: { 'Access-Control-Allow-Origin': '*' },
        });
        sql = `UPDATE ${table} SET ${column} = ? WHERE id = ?`;
        params = [wrap_param(value), wrap_param(id)];
        break;
      case 'delete':
        if (!id) return new Response('Missing id', {
          status: 400,
          headers: { 'Access-Control-Allow-Origin': '*' },
        });
        sql = `DELETE FROM ${table} WHERE id = ?`;
        params = [wrap_param(id)];
        break;
      // Helper to wrap parameter values for Turso/libSQL
      function wrap_param(val) {
        if (val === null || val === undefined) return { type: 'null', value: null };
        if (typeof val === 'number') {
          if (Number.isInteger(val)) return { type: 'integer', value: String(val) };
          return { type: 'real', value: String(val) };
        }
        if (typeof val === 'boolean') return { type: 'integer', value: val ? '1' : '0' };
        return { type: 'text', value: String(val) };
      }
      default:
        return new Response('Unknown action', {
          status: 400,
          headers: { 'Access-Control-Allow-Origin': '*' },
        });
    }
    // Call Turso/libSQL HTTP API
    const TURSO_URL = env.TURSO_URL;
    const TURSO_AUTH_TOKEN = env.TURSO_AUTH_TOKEN;
    if (!TURSO_URL || !TURSO_AUTH_TOKEN) {
      console.error('Missing DB credentials');
      return new Response('Missing DB credentials', {
        status: 500,
        headers: { 'Access-Control-Allow-Origin': '*' },
      });
    }
    const dbReq = [
      {
        steps: [
          { stmt: { sql, args: params } }
        ]
      }
    ];
    const resp = await fetch(`${TURSO_URL}/v1/batch`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${TURSO_AUTH_TOKEN}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(dbReq)
    });
    let dbRes;
    if (!resp.ok) {
      let errText = await resp.text();
      console.error('DB error response:', errText);
      return new Response(`DB error: ${errText}`, {
        status: 500,
        headers: { 'Access-Control-Allow-Origin': '*' },
      });
    }
    try {
      dbRes = await resp.json();
    } catch (e) {
      console.error('Failed to parse DB response as JSON:', e);
      return new Response(`DB error: Invalid JSON response`, {
        status: 500,
        headers: { 'Access-Control-Allow-Origin': '*' },
      });
    }
    // Format response for client
    // Always add CORS header to JSON responses
    const cors = { headers: { 'Access-Control-Allow-Origin': '*' } };
    switch (action) {
      case 'fetch': {
        // Transform step_results into array of objects
        const step = dbRes.result?.step_results?.[0];
        if (!step || !step.cols || !step.rows) {
          return new Response(JSON.stringify({ records: [] }), { ...cors, status: 200 });
        }
        const cols = step.cols;
        const records = step.rows.map(rowArr => {
          const obj = {};
          for (let i = 0; i < cols.length; i++) {
            const col = cols[i];
            let val = rowArr[i]?.value ?? null;
            // Convert to number if decltype is integer/real/float and value is string
            if (val !== null && typeof val === 'string' && col.decltype) {
              const decl = col.decltype.toUpperCase();
              if (decl.includes('INT')) {
                const parsed = parseInt(val, 10);
                if (!isNaN(parsed)) val = parsed;
              } else if (decl.includes('REAL') || decl.includes('FLOAT')) {
                const parsed = parseFloat(val);
                if (!isNaN(parsed)) val = parsed;
              }
            }
            obj[col.name] = val;
          }
          return obj;
        });
        return new Response(JSON.stringify({ records }), { ...cors, status: 200 });
      }
      case 'fetch_by_id':
        return new Response(JSON.stringify({ record: dbRes.results?.[0]?.results?.[0] ?? null }), { ...cors, status: 200 });
      case 'insert':
        return new Response(JSON.stringify({ id: dbRes.results?.[0]?.last_insert_rowid ?? 0 }), { ...cors, status: 200 });
      case 'update':
      case 'update_by_column':
        return new Response(JSON.stringify({ updated: dbRes.results?.[0]?.rows_affected ?? 0 }), { ...cors, status: 200 });
      case 'delete':
        return new Response(JSON.stringify({ deleted: dbRes.results?.[0]?.rows_affected ?? 0 }), { ...cors, status: 200 });
      default:
        return new Response(JSON.stringify({}), { ...cors, status: 200 });
    }
  }
};
