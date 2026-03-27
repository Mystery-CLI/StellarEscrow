#!/usr/bin/env node
/**
 * Stakeholder Feedback Collector
 *
 * Reads a feedback JSON file (submitted by stakeholders) and appends it
 * to the UAT feedback log. Generates a feedback summary report.
 *
 * Feedback file format (feedback-input.json):
 * {
 *   "stakeholder": "Product Owner",
 *   "name": "[name]",
 *   "uat_run": "2026-03-27T...",
 *   "overall_decision": "accept" | "reject" | "conditional",
 *   "items": [
 *     { "ac": "AC-1", "decision": "accept", "notes": "Works as expected" },
 *     { "ac": "AC-5", "decision": "reject", "notes": "Risk score not visible in UI" }
 *   ],
 *   "general_notes": "..."
 * }
 *
 * Usage: node scripts/collect-feedback.js [--input feedback-input.json]
 */

const fs = require('fs');
const path = require('path');

const args = process.argv.slice(2);
const getArg = (flag) => { const i = args.indexOf(flag); return i !== -1 ? args[i + 1] : null; };

const reportsDir = path.join(__dirname, '..', 'reports');
const inputPath  = getArg('--input') || path.join(reportsDir, 'feedback-input.json');
const logPath    = path.join(reportsDir, 'feedback-log.json');

if (!fs.existsSync(inputPath)) {
  console.log('[feedback] No feedback-input.json found.');
  console.log('[feedback] Create one with the format described in this script\'s header.');
  process.exit(0);
}

const input = JSON.parse(fs.readFileSync(inputPath, 'utf-8'));
input.received_at = new Date().toISOString();

// Append to log
let log = [];
if (fs.existsSync(logPath)) {
  log = JSON.parse(fs.readFileSync(logPath, 'utf-8'));
}
log.push(input);
fs.writeFileSync(logPath, JSON.stringify(log, null, 2));

// Generate summary
const summary = {
  generated_at: new Date().toISOString(),
  total_responses: log.length,
  decisions: { accept: 0, reject: 0, conditional: 0 },
  ac_feedback: {},
};

for (const entry of log) {
  summary.decisions[entry.overall_decision] = (summary.decisions[entry.overall_decision] || 0) + 1;
  for (const item of (entry.items || [])) {
    if (!summary.ac_feedback[item.ac]) {
      summary.ac_feedback[item.ac] = { accept: 0, reject: 0, conditional: 0, notes: [] };
    }
    summary.ac_feedback[item.ac][item.decision] = (summary.ac_feedback[item.ac][item.decision] || 0) + 1;
    if (item.notes) summary.ac_feedback[item.ac].notes.push(item.notes);
  }
}

fs.writeFileSync(path.join(reportsDir, 'feedback-summary.json'), JSON.stringify(summary, null, 2));

console.log('[feedback] Feedback recorded.');
console.log(`[feedback] Total responses: ${summary.total_responses}`);
console.log(`[feedback] Decisions: Accept=${summary.decisions.accept} Reject=${summary.decisions.reject} Conditional=${summary.decisions.conditional}`);
