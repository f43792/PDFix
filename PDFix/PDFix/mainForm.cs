using System.ComponentModel;

namespace PDFix
{
    public partial class MainForm : Form
    {
        public Dictionary<string, string> rulesDic = new Dictionary<string, string>();
        public BindingList<KeyValuePair<string, string>> itens; // = new BindingList<KeyValuePair<string, string>>(dic.ToList());
        public MainForm()
        {
            InitializeComponent();
        }

        private void exitToolStripMenuItem_Click(object sender, EventArgs e)
        {
            Application.Exit();
        }

        private void MainForm_Load(object sender, EventArgs e)
        {
            rulesDic.Add("1", "1.3");
            rulesDic.Add("2", "0.45");
            rulesDic.Add(">3", "1.5");

            itens  = new BindingList<KeyValuePair<string, string>>(rulesDic.ToList());
            dataGridView1.DataSource = itens;

            //dataGridView1.Columns.Clear();
            //dataGridView1.Columns.Add()
            dataGridView1.Columns[0].HeaderText = "Page";
            dataGridView1.Columns[1].HeaderText = "Line Width";
        }
    }
}
