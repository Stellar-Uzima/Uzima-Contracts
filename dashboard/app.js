document.addEventListener('DOMContentLoaded', () => {
    // Mock Data for Demonstration
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

    // Initialize UI
    updateStats();
    populateTable();
    initCharts();

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
        // Line Chart for Calls
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
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: false }
                },
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
            }
        });

        // Pie Chart for Function Distribution
        const funcCtx = document.getElementById('functionChart').getContext('2d');
        new Chart(funcCtx, {
            type: 'doughnut',
            data: {
                labels: mockFunctions.map(f => f.name),
                datasets: [{
                    data: mockFunctions.map(f => f.calls),
                    backgroundColor: [
                        '#6366f1',
                        '#a855f7',
                        '#ec4899',
                        '#10b981',
                        '#f59e0b'
                    ],
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
                        labels: {
                            color: '#94a3b8',
                            padding: 20,
                            usePointStyle: true,
                            font: { size: 10 }
                        }
                    }
                },
                cutout: '70%'
            }
        });
    }

    // Refresh Button Event
    document.getElementById('refresh-btn').addEventListener('click', () => {
        const btn = document.getElementById('refresh-btn');
        btn.textContent = 'Refreshing...';
        btn.disabled = true;
        
        setTimeout(() => {
            btn.textContent = 'Refresh Data';
            btn.disabled = false;
            // In a real app, this would re-fetch from the contract
        }, 1000);
    });
});
