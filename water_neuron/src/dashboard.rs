use crate::read_state;
use crate::state::{ICP_LEDGER_ID, NNS_GOVERNANCE_ID};
use candid::Principal;
use std::fmt;
use std::io::Write;

pub fn build_dashboard() -> Vec<u8> {
    format!(
        "
    <!DOCTYPE html>
    <html lang=\"en\">
        <head>
            <title>WaterNeuron Dashboard</title>
            <style>
                body {{
                    font-family: monospace;
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                }}
                table {{
                    border: none;
                    text-align: left;
                    border-width: thin;
                    border-collapse: collapse;
                    width: 100%;
                }}
                .table-container th {{
                    position: sticky;
                    top: 0; 
                    z-index: 10; 
                    background: #ffffff;
                    height: 40px;
                }}
                th, td {{
                    padding: 5px 10px;
                }}
                h3 {{
                    font-variant: small-caps;
                    margin-top: 40px;
                }}
                li {{
                    display: flex; 
                    flex-direction: row; 
                    align-items: center; 
                    gap: 1em;
                }}
                .table-container {{
                    max-height: 350px; 
                    overflow: auto;
                    border: 1px solid black;
                    width: 100%;
                    overflow-y: auto;
                    min-width: 400px;
                }}

                .metadata-container {{
                    margin: 0 auto;
                    max-width: fit-content;
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                }}

                .metadata-container table {{
                    border: 1px solid black;
                }}

                .metadata-container th, .metadata-container td {{
                    padding: 3px;
                }}
                body::-webkit-scrollbar {{
                    display: none; 
                }}

                tbody tr:nth-child(odd) {{ background-color: #eeeeee; }} 
            </style>
            <script>
                document.addEventListener(\"DOMContentLoaded\", function() {{
                    var tds = document.querySelectorAll(\".ts-class\");
                    for (var i = 0; i < tds.length; i++) {{
                    var td = tds[i];
                    var timestamp = td.textContent / 1000000;
                    var date = new Date(timestamp);
                    var options = {{
                        year: 'numeric',
                        month: 'short',
                        day: 'numeric',
                        hour: 'numeric',
                        minute: 'numeric',
                        second: 'numeric'
                    }};
                    td.title = td.textContent;
                    td.textContent = date.toLocaleString(undefined, options);
                    }}
                }});
            </script>
        </head>
        <body>
            <div class=\"metadata-container\">
                <h3>Metadata</h3>
                {}
            </div>
            <div style=\"width: 100%;\">
                <h3>Tasks</h3>
                <ul>{}</ul>
            </div>
            <h3>Withdrawal Requests</h3>
            <div class=\"table-container\">
                <table>
                    <thead>
                        <tr>
                            <th>Withdrawal Id</th>
                            <th>Received at</th>
                            <th>Receiver</th>
                            <th>nICP Burned Amount</th>
                            <th>nICP Burn Index</th>
                            <th>ICP Due Amount</th>
                            <th>Status</th>
                            <th>Neuron ID</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
            <h3>Deposits History</h3>
            <div class=\"table-container\">
                <table>
                    <thead>
                        <tr>
                            <th>Transfer Id</th>
                            <th>Receiver</th>
                            <th>Amount (nICP)</th>
                            <th>Block Index</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
            <h3>Maturity Neuron Spawned</h3>
            <div class=\"table-container\">

                <table>
                    <thead>
                        <tr>
                            <th>Neuron Id</th>
                            <th>Block Index</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
            <h3>Spawned Neurons</h3>
            <div class=\"table-container\">
                <table>
                    <thead>
                        <tr>
                            <th>Neuron Id</th>
                            <th>Receiver</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
            {}
        </div>
    </body>
</html>
    ",
        construct_metadata_table(),
        display_tasks(),
        construct_withdrawal_table(),
        construct_deposit_table(),
        construct_maturity_neuron_table(),
        construct_to_disburse_table(),
        get_pending_transfer_table(),
    )
    .into_bytes()
}

fn with_utf8_buffer(f: impl FnOnce(&mut Vec<u8>)) -> String {
    let mut buf = Vec::new();
    f(&mut buf);
    String::from_utf8(buf).unwrap()
}

fn construct_maturity_neuron_table() -> String {
    with_utf8_buffer(|buf| {
        read_state(|s| {
            for (neuron_id, block_index) in s.maturity_neuron_to_block_indicies.iter() {
                write!(
                    buf,
                    "<tr><td><a href=\"{}\" target=\"_blank\">{}</a></td><td>{}</td></tr>",
                    neuron_id.to_dashboard_link(),
                    neuron_id.id,
                    block_index
                )
                .unwrap();
            }
        });
    })
}

fn construct_metadata_table() -> String {
    read_state(|s| {
        format!(
            "<table>
                <tbody>
                    <tr>
                        <th>ICP Ledger Principal</th>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <th>nICP Ledger Principal</th>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <th>NNS Governance Principal</th>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <th>SNS Governance Principal</th>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <th>WTN Ledger Principal</th>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <th rowspan=\"4\">6-month nICP Neuron</th>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>Fetched {} ICP </td>
                    </tr>
                    <tr>
                        <td>Tracked {} ICP</td>
                    </tr>
                    <tr>
                        <td>
                            <a target=\"_blank\" href=\"https://dashboard.internetcomputer.org/account/11db16e8da65784bc03feafc67fde5317bbbe59852bd6d9af8f8753e8c8e3279\">Maturity Account</a>
                        </td>
                    </tr>
                    <tr>
                        <th rowspan=\"3\">8-year SNS Neuron</th>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>Fetched {} ICP</td>
                    </tr>
                    <tr>
                        <td>
                            <a target=\"_blank\" href=\"https://dashboard.internetcomputer.org/account/45075f09714a20a804a3f4ab4b7d9d45c1b93b500b525d5a9b8bfa394e2765d4\">Maturity Account</a>
                        </td>
                    </tr>
                    <tr>
                        <th>SNS Rewards</th>
                        <td>
                            <a target=\"_blank\" href=\"https://dashboard.internetcomputer.org/account/ff6987e51d16f29ffc4c7932b8e2e93e558fc617732fd17de4db9f5a43c9994c\">SNS Rewards Account</a>
                        </td>
                    </tr>
                    <tr>
                        <th>nICP Circulating</th>
                        <td>{} nICP</td>
                    </tr>
                    <tr>
                        <th>Stakers Count</th>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <th>Exchange Rate</th>
                        <td>{}</td>
                    </tr>
                </tbody>
            </table>",
            link_to_dashboard(ICP_LEDGER_ID),
            link_to_dashboard(s.nicp_ledger_id),
            link_to_dashboard(NNS_GOVERNANCE_ID),
            link_to_dashboard(s.wtn_governance_id),
            link_to_dashboard(s.wtn_ledger_id),
            s.neuron_id_6m
                .map(|n| format!(
                    "<a href=\"{}\" target=\"_blank\">{}</a>",
                    n.to_dashboard_link(),
                    n.id
                ))
                .unwrap_or_else(|| "Neuron Not Set".to_string()),
            s.main_neuron_6m_staked,
            s.tracked_6m_stake,
            s.neuron_id_8y
                .map(|n| format!(
                    "<a href=\"{}\" target=\"_blank\">{}</a>",
                    n.to_dashboard_link(),
                    n.id
                ))
                .unwrap_or_else(|| "Neuron Not Set".to_string()),
            s.main_neuron_8y_stake,
            s.total_circulating_nicp,
            s.account_to_deposits.keys().len(),
            s.get_icp_to_ncip_exchange_rate_e8s()
        )
    })
}

fn link_to_dashboard(principal: Principal) -> String {
    format!("<a href=\"https://dashboard.internetcomputer.org/canister/{principal}\" target=\"_blank\">{principal}</a>")
}

fn construct_withdrawal_table() -> String {
    with_utf8_buffer(|buf| {
        read_state(|s| {
            for req in s.withdrawal_id_to_request.values() {
                write!(
                    buf,
                    "<tr><td>{}</td><td class=\"ts-class\">{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    req.withdrawal_id,
                    req.timestamp,
                    req.receiver,
                    req.nicp_burned,
                    req.nicp_burn_index,
                    req.icp_due,
                    s.get_withdrawal_status(req.withdrawal_id),
                    req.neuron_id
                        .map(|n| format!(
                            "<a href=\"{}\" target=\"_blank\">{}</a>",
                            n.to_dashboard_link(),
                            n.id
                        ))
                        .unwrap_or_else(|| "Neuron Not Set".to_string())
                )
                .unwrap();
            }
        });
    })
}

fn construct_deposit_table() -> String {
    with_utf8_buffer(|buf| {
        read_state(|s| {
            for (account, transfer_ids) in s.account_to_deposits.iter() {
                for transfer_id in transfer_ids {
                    if let Some(deposit) = s.transfer_executed.get(transfer_id) {
                        write!(
                            buf,
                            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:?}</td></tr>",
                            deposit.transfer.transfer_id,
                            account,
                            DisplayAmount(deposit.transfer.amount),
                            deposit.block_index
                        )
                        .unwrap();
                    }
                }
            }
        });
    })
}

fn construct_to_disburse_table() -> String {
    with_utf8_buffer(|buf| {
        read_state(|s| {
            for (neuron_id, disburse_request) in s.to_disburse.iter() {
                let neuron_string = format!(
                    "<a href=\"{}\" target=\"_blank\">{}</a>",
                    neuron_id.to_dashboard_link(),
                    neuron_id.id
                );
                write!(
                    buf,
                    "<tr><td>{}</td><td>{}</td></tr>",
                    neuron_string, disburse_request.receiver,
                )
                .unwrap();
            }
        });
    })
}

fn get_pending_transfer_table() -> String {
    with_utf8_buffer(|buf| {
        read_state(|s| {
            if !s.pending_transfers.is_empty() {
                write!(
                    buf,
                    "
                    <h3>Pending Transfers</h3>
                    <div class=\"table-container\">
                        <table>
                            <thead>
                                <tr>
                                    <th>Transfer Id</th>
                                    <th>Receiver</th>
                                    <th>Amount</th>
                                    <th>Unit</th>
                                </tr>
                            </thead>
                            <tbody>
                                {}
                            </tbody>
                        </table>
                    </div>
                ",
                    construct_pending_transfer_table()
                )
                .unwrap();
            }
        });
    })
}

fn construct_pending_transfer_table() -> String {
    with_utf8_buffer(|buf| {
        read_state(|s| {
            for transfer in s.pending_transfers.values() {
                write!(
                    buf,
                    "
                <tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>
                ",
                    transfer.transfer_id,
                    transfer.receiver,
                    DisplayAmount(transfer.amount),
                    transfer.unit,
                )
                .unwrap();
            }
            write!(
                buf,
                "<tr><td colspan='3' style='text-align: right;'><b>Pending Transfers Count</b></td><td>{}</td></tr>",
                s.pending_transfers.len()
            )
            .unwrap();
        });
    })
}

fn display_tasks() -> String {
    with_utf8_buffer(|buf| {
        let tasks = crate::tasks::get_task_queue();
        for task in tasks {
            write!(
                buf,
                "<li style=\"display: flex; flex-direction: row; align-items: center; gap: 1em;\">{:?} at <p class=\"ts-class\">{}</p></li>",
                task.task_type, task.execute_at
            )
            .unwrap()
        }
    })
}

pub struct DisplayAmount(pub u64);

impl fmt::Display for DisplayAmount {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        const E8S: u64 = 100_000_000;
        let int = self.0 / E8S;
        let frac = self.0 % E8S;

        if frac > 0 {
            let frac_width: usize = {
                // Count decimal digits in the fraction part.
                let mut d = 0;
                let mut x = frac;
                while x > 0 {
                    d += 1;
                    x /= 10;
                }
                d
            };
            debug_assert!(frac_width <= 8);
            let frac_prefix: u64 = {
                // The fraction part without trailing zeros.
                let mut f = frac;
                while f % 10 == 0 {
                    f /= 10
                }
                f
            };

            write!(fmt, "{}.", int)?;
            for _ in 0..(8 - frac_width) {
                write!(fmt, "0")?;
            }
            write!(fmt, "{}", frac_prefix)
        } else {
            write!(fmt, "{}.0", int)
        }
    }
}
