namespace PDFix
{
    partial class MainForm
    {
        /// <summary>
        ///  Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        ///  Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        /// <summary>
        ///  Required method for Designer support - do not modify
        ///  the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            lateralPanel = new Panel();
            groupBox1 = new GroupBox();
            dataGridView1 = new DataGridView();
            radioButton4 = new RadioButton();
            textBox2 = new TextBox();
            label5 = new Label();
            textBox1 = new TextBox();
            label4 = new Label();
            radioButton3 = new RadioButton();
            radioButton2 = new RadioButton();
            radioButton1 = new RadioButton();
            gbHairlines = new GroupBox();
            comboBox1 = new ComboBox();
            label3 = new Label();
            checkBox2 = new CheckBox();
            checkBox1 = new CheckBox();
            numericUpDown2 = new NumericUpDown();
            label2 = new Label();
            numericUpDown1 = new NumericUpDown();
            label1 = new Label();
            mainPanel = new Panel();
            button1 = new Button();
            upperPanel = new Panel();
            mainMenu = new MenuStrip();
            fileToolStripMenuItem = new ToolStripMenuItem();
            toolStripMenuItem1 = new ToolStripSeparator();
            exitToolStripMenuItem = new ToolStripMenuItem();
            fbd = new FolderBrowserDialog();
            basePanel = new Panel();
            splitter1 = new Splitter();
            helpToolStripMenuItem = new ToolStripMenuItem();
            howToToolStripMenuItem = new ToolStripMenuItem();
            aboutToolStripMenuItem = new ToolStripMenuItem();
            lateralPanel.SuspendLayout();
            groupBox1.SuspendLayout();
            ((System.ComponentModel.ISupportInitialize)dataGridView1).BeginInit();
            gbHairlines.SuspendLayout();
            ((System.ComponentModel.ISupportInitialize)numericUpDown2).BeginInit();
            ((System.ComponentModel.ISupportInitialize)numericUpDown1).BeginInit();
            mainPanel.SuspendLayout();
            mainMenu.SuspendLayout();
            basePanel.SuspendLayout();
            SuspendLayout();
            // 
            // lateralPanel
            // 
            lateralPanel.BackColor = SystemColors.ButtonFace;
            lateralPanel.Controls.Add(groupBox1);
            lateralPanel.Controls.Add(gbHairlines);
            lateralPanel.Dock = DockStyle.Left;
            lateralPanel.Location = new Point(0, 0);
            lateralPanel.Margin = new Padding(16);
            lateralPanel.Name = "lateralPanel";
            lateralPanel.Padding = new Padding(16);
            lateralPanel.Size = new Size(418, 737);
            lateralPanel.TabIndex = 0;
            // 
            // groupBox1
            // 
            groupBox1.Controls.Add(dataGridView1);
            groupBox1.Controls.Add(radioButton4);
            groupBox1.Controls.Add(textBox2);
            groupBox1.Controls.Add(label5);
            groupBox1.Controls.Add(textBox1);
            groupBox1.Controls.Add(label4);
            groupBox1.Controls.Add(radioButton3);
            groupBox1.Controls.Add(radioButton2);
            groupBox1.Controls.Add(radioButton1);
            groupBox1.Dock = DockStyle.Top;
            groupBox1.Location = new Point(16, 253);
            groupBox1.Margin = new Padding(8, 32, 8, 8);
            groupBox1.Name = "groupBox1";
            groupBox1.Padding = new Padding(8, 16, 8, 8);
            groupBox1.Size = new Size(386, 373);
            groupBox1.TabIndex = 1;
            groupBox1.TabStop = false;
            groupBox1.Text = "Page Range";
            // 
            // dataGridView1
            // 
            dataGridView1.BorderStyle = BorderStyle.None;
            dataGridView1.ColumnHeadersHeightSizeMode = DataGridViewColumnHeadersHeightSizeMode.AutoSize;
            dataGridView1.EditMode = DataGridViewEditMode.EditOnEnter;
            dataGridView1.Location = new Point(22, 163);
            dataGridView1.Name = "dataGridView1";
            dataGridView1.Size = new Size(328, 150);
            dataGridView1.TabIndex = 14;
            // 
            // radioButton4
            // 
            radioButton4.AutoSize = true;
            radioButton4.Location = new Point(27, 128);
            radioButton4.Name = "radioButton4";
            radioButton4.Size = new Size(99, 19);
            radioButton4.TabIndex = 13;
            radioButton4.Text = "Use page rule:";
            radioButton4.UseVisualStyleBackColor = true;
            // 
            // textBox2
            // 
            textBox2.Location = new Point(294, 96);
            textBox2.Name = "textBox2";
            textBox2.Size = new Size(56, 23);
            textBox2.TabIndex = 12;
            // 
            // label5
            // 
            label5.AutoSize = true;
            label5.Location = new Point(261, 99);
            label5.Name = "label5";
            label5.Size = new Size(23, 15);
            label5.TabIndex = 11;
            label5.Text = "To:";
            label5.TextAlign = ContentAlignment.MiddleRight;
            // 
            // textBox1
            // 
            textBox1.Location = new Point(195, 96);
            textBox1.Name = "textBox1";
            textBox1.Size = new Size(56, 23);
            textBox1.TabIndex = 10;
            // 
            // label4
            // 
            label4.AutoSize = true;
            label4.Location = new Point(141, 96);
            label4.Name = "label4";
            label4.Size = new Size(38, 15);
            label4.TabIndex = 9;
            label4.Text = "From:";
            label4.TextAlign = ContentAlignment.MiddleRight;
            // 
            // radioButton3
            // 
            radioButton3.AutoSize = true;
            radioButton3.Location = new Point(27, 92);
            radioButton3.Name = "radioButton3";
            radioButton3.Size = new Size(92, 19);
            radioButton3.TabIndex = 2;
            radioButton3.Text = "Pages range:";
            radioButton3.UseVisualStyleBackColor = true;
            // 
            // radioButton2
            // 
            radioButton2.AutoSize = true;
            radioButton2.Checked = true;
            radioButton2.Location = new Point(27, 63);
            radioButton2.Name = "radioButton2";
            radioButton2.Size = new Size(94, 19);
            radioButton2.TabIndex = 1;
            radioButton2.TabStop = true;
            radioButton2.Text = "Current Page";
            radioButton2.UseVisualStyleBackColor = true;
            // 
            // radioButton1
            // 
            radioButton1.AutoSize = true;
            radioButton1.Location = new Point(27, 35);
            radioButton1.Name = "radioButton1";
            radioButton1.Size = new Size(144, 19);
            radioButton1.TabIndex = 0;
            radioButton1.Text = "All Pages in document";
            radioButton1.UseVisualStyleBackColor = true;
            // 
            // gbHairlines
            // 
            gbHairlines.Controls.Add(comboBox1);
            gbHairlines.Controls.Add(label3);
            gbHairlines.Controls.Add(checkBox2);
            gbHairlines.Controls.Add(checkBox1);
            gbHairlines.Controls.Add(numericUpDown2);
            gbHairlines.Controls.Add(label2);
            gbHairlines.Controls.Add(numericUpDown1);
            gbHairlines.Controls.Add(label1);
            gbHairlines.Dock = DockStyle.Top;
            gbHairlines.Location = new Point(16, 16);
            gbHairlines.Margin = new Padding(16);
            gbHairlines.Name = "gbHairlines";
            gbHairlines.Padding = new Padding(16);
            gbHairlines.Size = new Size(386, 237);
            gbHairlines.TabIndex = 0;
            gbHairlines.TabStop = false;
            gbHairlines.Text = "Hairlines";
            // 
            // comboBox1
            // 
            comboBox1.DisplayMember = "0";
            comboBox1.FormattingEnabled = true;
            comboBox1.Items.AddRange(new object[] { "Points", "Picas", "Milimiters", "Centimeters", "Inches" });
            comboBox1.Location = new Point(191, 133);
            comboBox1.Name = "comboBox1";
            comboBox1.Size = new Size(121, 23);
            comboBox1.TabIndex = 7;
            comboBox1.ValueMember = "0";
            // 
            // label3
            // 
            label3.AutoSize = true;
            label3.Location = new Point(102, 136);
            label3.Name = "label3";
            label3.Size = new Size(34, 15);
            label3.TabIndex = 6;
            label3.Text = "Units";
            label3.TextAlign = ContentAlignment.MiddleRight;
            // 
            // checkBox2
            // 
            checkBox2.AutoSize = true;
            checkBox2.Location = new Point(191, 182);
            checkBox2.Name = "checkBox2";
            checkBox2.Size = new Size(111, 19);
            checkBox2.TabIndex = 5;
            checkBox2.Text = "Include Patterns";
            checkBox2.UseVisualStyleBackColor = true;
            // 
            // checkBox1
            // 
            checkBox1.AutoSize = true;
            checkBox1.Location = new Point(42, 182);
            checkBox1.Name = "checkBox1";
            checkBox1.Size = new Size(129, 19);
            checkBox1.TabIndex = 4;
            checkBox1.Text = "Include Type3 fonts";
            checkBox1.UseVisualStyleBackColor = true;
            // 
            // numericUpDown2
            // 
            numericUpDown2.Location = new Point(191, 93);
            numericUpDown2.Name = "numericUpDown2";
            numericUpDown2.Size = new Size(120, 23);
            numericUpDown2.TabIndex = 3;
            // 
            // label2
            // 
            label2.AutoSize = true;
            label2.Location = new Point(102, 95);
            label2.Name = "label2";
            label2.Size = new Size(74, 15);
            label2.TabIndex = 2;
            label2.Text = "Replace with";
            label2.TextAlign = ContentAlignment.MiddleRight;
            // 
            // numericUpDown1
            // 
            numericUpDown1.Location = new Point(191, 52);
            numericUpDown1.Name = "numericUpDown1";
            numericUpDown1.Size = new Size(120, 23);
            numericUpDown1.TabIndex = 1;
            // 
            // label1
            // 
            label1.AutoSize = true;
            label1.Location = new Point(33, 54);
            label1.Name = "label1";
            label1.Size = new Size(143, 15);
            label1.TabIndex = 0;
            label1.Text = "Narrower than or equal to";
            // 
            // mainPanel
            // 
            mainPanel.BackColor = Color.Azure;
            mainPanel.Controls.Add(button1);
            mainPanel.Controls.Add(upperPanel);
            mainPanel.Dock = DockStyle.Fill;
            mainPanel.Location = new Point(421, 0);
            mainPanel.Name = "mainPanel";
            mainPanel.Size = new Size(763, 737);
            mainPanel.TabIndex = 2;
            // 
            // button1
            // 
            button1.Location = new Point(20, 118);
            button1.Name = "button1";
            button1.Size = new Size(134, 31);
            button1.TabIndex = 1;
            button1.Text = "button1";
            button1.UseVisualStyleBackColor = true;
            // 
            // upperPanel
            // 
            upperPanel.BackColor = Color.RosyBrown;
            upperPanel.Dock = DockStyle.Top;
            upperPanel.Location = new Point(0, 0);
            upperPanel.Name = "upperPanel";
            upperPanel.Size = new Size(763, 97);
            upperPanel.TabIndex = 0;
            // 
            // mainMenu
            // 
            mainMenu.Items.AddRange(new ToolStripItem[] { fileToolStripMenuItem, helpToolStripMenuItem });
            mainMenu.Location = new Point(0, 0);
            mainMenu.Name = "mainMenu";
            mainMenu.Size = new Size(1184, 24);
            mainMenu.TabIndex = 3;
            mainMenu.Text = "menuStrip1";
            // 
            // fileToolStripMenuItem
            // 
            fileToolStripMenuItem.DropDownItems.AddRange(new ToolStripItem[] { toolStripMenuItem1, exitToolStripMenuItem });
            fileToolStripMenuItem.Name = "fileToolStripMenuItem";
            fileToolStripMenuItem.Size = new Size(37, 20);
            fileToolStripMenuItem.Text = "File";
            // 
            // toolStripMenuItem1
            // 
            toolStripMenuItem1.Name = "toolStripMenuItem1";
            toolStripMenuItem1.Size = new Size(89, 6);
            // 
            // exitToolStripMenuItem
            // 
            exitToolStripMenuItem.Name = "exitToolStripMenuItem";
            exitToolStripMenuItem.Size = new Size(92, 22);
            exitToolStripMenuItem.Text = "Exit";
            // 
            // basePanel
            // 
            basePanel.Controls.Add(mainPanel);
            basePanel.Controls.Add(splitter1);
            basePanel.Controls.Add(lateralPanel);
            basePanel.Dock = DockStyle.Fill;
            basePanel.Location = new Point(0, 24);
            basePanel.Name = "basePanel";
            basePanel.Size = new Size(1184, 737);
            basePanel.TabIndex = 1;
            // 
            // splitter1
            // 
            splitter1.Location = new Point(418, 0);
            splitter1.Name = "splitter1";
            splitter1.Size = new Size(3, 737);
            splitter1.TabIndex = 3;
            splitter1.TabStop = false;
            // 
            // helpToolStripMenuItem
            // 
            helpToolStripMenuItem.DropDownItems.AddRange(new ToolStripItem[] { howToToolStripMenuItem, aboutToolStripMenuItem });
            helpToolStripMenuItem.Name = "helpToolStripMenuItem";
            helpToolStripMenuItem.Size = new Size(44, 20);
            helpToolStripMenuItem.Text = "Help";
            // 
            // howToToolStripMenuItem
            // 
            howToToolStripMenuItem.Name = "howToToolStripMenuItem";
            howToToolStripMenuItem.Size = new Size(180, 22);
            howToToolStripMenuItem.Text = "How to";
            // 
            // aboutToolStripMenuItem
            // 
            aboutToolStripMenuItem.Name = "aboutToolStripMenuItem";
            aboutToolStripMenuItem.Size = new Size(180, 22);
            aboutToolStripMenuItem.Text = "About...";
            // 
            // MainForm
            // 
            AutoScaleDimensions = new SizeF(7F, 15F);
            AutoScaleMode = AutoScaleMode.Font;
            ClientSize = new Size(1184, 761);
            Controls.Add(basePanel);
            Controls.Add(mainMenu);
            DoubleBuffered = true;
            FormBorderStyle = FormBorderStyle.FixedSingle;
            Name = "MainForm";
            StartPosition = FormStartPosition.CenterScreen;
            Text = "PDFix";
            Load += MainForm_Load;
            lateralPanel.ResumeLayout(false);
            groupBox1.ResumeLayout(false);
            groupBox1.PerformLayout();
            ((System.ComponentModel.ISupportInitialize)dataGridView1).EndInit();
            gbHairlines.ResumeLayout(false);
            gbHairlines.PerformLayout();
            ((System.ComponentModel.ISupportInitialize)numericUpDown2).EndInit();
            ((System.ComponentModel.ISupportInitialize)numericUpDown1).EndInit();
            mainPanel.ResumeLayout(false);
            mainMenu.ResumeLayout(false);
            mainMenu.PerformLayout();
            basePanel.ResumeLayout(false);
            ResumeLayout(false);
            PerformLayout();
        }

        #endregion

        private Panel lateralPanel;
        private Panel mainPanel;
        private Panel upperPanel;
        private MenuStrip mainMenu;
        private ToolStripMenuItem fileToolStripMenuItem;
        private ToolStripSeparator toolStripMenuItem1;
        private ToolStripMenuItem exitToolStripMenuItem;
        private FolderBrowserDialog fbd;
        private Panel basePanel;
        private Splitter splitter1;
        private Button button1;
        private GroupBox gbHairlines;
        private NumericUpDown numericUpDown2;
        private Label label2;
        private NumericUpDown numericUpDown1;
        private Label label1;
        private CheckBox checkBox2;
        private CheckBox checkBox1;
        private ComboBox comboBox1;
        private Label label3;
        private GroupBox groupBox1;
        private RadioButton radioButton1;
        private TextBox textBox2;
        private Label label5;
        private TextBox textBox1;
        private Label label4;
        private RadioButton radioButton3;
        private RadioButton radioButton2;
        private RadioButton radioButton4;
        private DataGridView dataGridView1;
        private ToolStripMenuItem helpToolStripMenuItem;
        private ToolStripMenuItem howToToolStripMenuItem;
        private ToolStripMenuItem aboutToolStripMenuItem;
    }
}
