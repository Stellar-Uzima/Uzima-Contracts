document.addEventListener('DOMContentLoaded', () => {
    const mockFunctions = [
        { name: 'submit_clinical_trial', calls: 1250, cpu: '45.2M', ram: '1.2MB', error: '0.5%', latency: '120ms' },
        { name: 'authorize_access', calls: 3420, cpu: '12.8M', ram: '0.4MB', error: '0.1%', latency: '45ms' },
        { name: 'register_patient', calls: 890, cpu: '85.4M', ram: '2.5MB', error: '1.2%', latency: '210ms' },
        { name: 'mint_consent_nft', calls: 450, cpu: '120.1M', ram: '4.8MB', error: '2.5%', latency: '350ms' },
        { name: 'sync_medical_record', calls: 2100, cpu: '65.2M', ram: '3.1MB', error: '0.8%', latency: '180ms' }
    ];

    const mockHistory = {
        labels: ['Apr 19', 'Apr 20', 'Apr 21', 'Apr 22', 'Apr 23', 'Apr 24', 'Apr 25'],
        calls: [450, 620, 580, 890, 750, 1100, 1250],
        activeUsers: [120, 150, 140, 210, 180, 250, 280]
    };

    let complexityTrendChart;

    updateStats();
    populateTable();
    initCharts();
    loadComplexityData();

    document.getElementById('refresh-btn').addEventListener('click', async () => {
        const btn = document.getElementById('refresh-btn');
        btn.textContent = 'Refreshing...';
        btn.disabled = true;
        await loadComplexityData();
        setTimeout(() => {
            btn.textContent = 'Refresh Data';
            btn.disabled = false;
        }, 600);
    });

    function updateStats() {
        document.getElementById('total-calls').textContent = '8,110';
        document.getElementById('active-users').textContent = '280';
        document.getElementById('error-rate').textContent = '0.85%';
        document.getElementById('avg-latency').textContent = '145ms';
    }

    function populateTable() {
        const tbody = document.querySelector('#functions-table tbody');
        tbody.innerHTML = '';

        mockFunctions.forEach(func => {
            const row = `
                <tr>
                    <td style="font-weight: 600;">${func.name}</td>
                    <td>${func.calls.toLocaleString()}</td>
                    <td>${func.cpu}</td>
                    <td>${func.ram}</td>
                    <td style="color: ${parseFloat(func.error) > 1 ? 'var(--danger)' : 'var(--success)'}">${func.error}</td>
                    <td>${func.latency}</td>
                </tr>
            `;
            tbody.innerHTML += row;
        });
    }

    function initCharts() {
        const callsCtx = document.getElementById('callsChart').getContext('2d');
        new Chart(callsCtx, {
            type: 'line',
            data: {
                labels: mockHistory.labels,
                datasets: [{
                    label: 'Contract Calls',
                    data: mockHistory.calls,
                    borderColor: '#6366f1',
                    backgroundColor: 'rgba(99, 102, 241, 0.1)',
                    fill: true,
                    tension: 0.4,
                    borderWidth: 3,
                    pointRadius: 4,
                    pointBackgroundColor: '#6366f1'
                }]
            },
            options: chartLineOptions()
        });

        const funcCtx = document.getElementById('functionChart').getContext('2d');
        new Chart(funcCtx, {
            type: 'doughnut',
            data: {
                labels: mockFunctions.map(f => f.name),
                datasets: [{
                    data: mockFunctions.map(f => f.calls),
                    backgroundColor: ['#6366f1', '#a855f7', '#ec4899', '#10b981', '#f59e0b'],
                    borderWidth: 0,
                    hoverOffset: 10
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        position: 'bottom',
                        labels: { color: '#94a3b8', padding: 20, usePointStyle: true, font: { size: 10 } }
                    }
                },
                cutout: '70%'
            }
        });
    }

    async function loadComplexityData() {
        const report = await fetchJson('data/complexity_report.json');
        const trends = await fetchJson('data/complexity_trends.json');

        if (!report || !report.contracts || report.contracts.length === 0) {
            document.getElementById('complexity-trend-label').textContent =
                'Run ./scripts/complexity_score.sh to generate scores';
            document.getElementById('complexity-generated').textContent =
                'No report found — run ./scripts/complexity_score.sh';
            return;
        }

        renderComplexitySummary(report, trends);
        renderComplexityTable(report.contracts);
        renderComplexityTrendChart(report, trends);
    }

    function renderComplexitySummary(report, trends) {
        const contracts = report.contracts;

        document.getElementById('complexity-avg').textContent = String(report.workspace_average);
        document.getElementById('complexity-count').textContent = String(contracts.length);

        const generated = report.generated_at
            ? new Date(Number(report.generated_at) * 1000).toLocaleString()
            : '—';
        document.getElementById('complexity-generated').textContent = `Updated ${generated}`;

        const trendLabel = document.getElementById('complexity-trend-label');
        if (trends && trends.snapshots && trends.snapshots.length >= 2) {
            const prev = trends.snapshots[trends.snapshots.length - 2].workspace_average;
            const delta = report.workspace_average - prev;
            const sign = delta > 0 ? '+' : '';
            trendLabel.textContent = `${sign}${delta} vs previous run`;
            trendLabel.className = `stat-trend ${delta > 0 ? 'negative' : delta < 0 ? 'positive' : 'neutral'}`;
        } else {
            trendLabel.textContent = 'Trend history grows on each script run';
            trendLabel.className = 'stat-trend neutral';
        }
    }

    function renderComplexityTable(contracts) {
        const tbody = document.querySelector('#complexity-table tbody');
        tbody.innerHTML = '';

        contracts.forEach(c => {
            const gradeClass = `grade-${String(c.grade).toLowerCase()}`;
            tbody.innerHTML += `
                <tr>
                    <td style="font-weight: 600;">${c.contract_name}</td>
                    <td>${c.total_score}</td>
                    <td><span class="grade-badge ${gradeClass}">${c.grade}</span></td>
                    <td>${c.component_scores.cyclomatic}</td>
                    <td>${c.component_scores.data_structure}</td>
                    <td>${c.component_scores.external_interaction}</td>
                    <td>${c.component_scores.state_transition}</td>
                    <td>${c.component_scores.permission_model}</td>
                </tr>
            `;
        });
    }

    function renderComplexityTrendChart(report, trends) {
        const snapshots = (trends && trends.snapshots) ? trends.snapshots : [];
        const labels = snapshots.map((s, i) => {
            if (s.recorded_at) {
                return new Date(Number(s.recorded_at) * 1000).toLocaleDateString();
            }
            return `Run ${i + 1}`;
        });
        const averages = snapshots.map(s => s.workspace_average);

        if (labels.length === 0 && report.workspace_average) {
            labels.push('Current');
            averages.push(report.workspace_average);
        }

        const trendCtx = document.getElementById('complexityTrendChart').getContext('2d');
        if (complexityTrendChart) complexityTrendChart.destroy();
        complexityTrendChart = new Chart(trendCtx, {
            type: 'line',
            data: {
                labels,
                datasets: [{
                    label: 'Workspace avg complexity',
                    data: averages,
                    borderColor: '#f59e0b',
                    backgroundColor: 'rgba(245, 158, 11, 0.1)',
                    fill: true,
                    tension: 0.35,
                    borderWidth: 2
                }]
            },
            options: chartLineOptions()
        });
    }

    function chartLineOptions() {
        return {
            responsive: true,
            maintainAspectRatio: false,
            plugins: { legend: { display: false } },
            scales: {
                y: {
                    beginAtZero: true,
                    grid: { color: 'rgba(255, 255, 255, 0.05)' },
                    ticks: { color: '#94a3b8' }
                },
                x: {
                    grid: { display: false },
                    ticks: { color: '#94a3b8' }
                }
            }
        };
    }

    async function fetchJson(path) {
        try {
            const res = await fetch(path);
            if (!res.ok) return null;
            return await res.json();
        } catch {
            return null;
        }
    }
});
