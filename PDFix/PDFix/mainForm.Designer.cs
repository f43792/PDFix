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
            mainPanel.SuspendLayout();
            mainMenu.SuspendLayout();
            basePanel.SuspendLayout();
            SuspendLayout();
            // 
            // lateralPanel
            // 
            lateralPanel.BackColor = Color.GreenYellow;
            lateralPanel.Dock = DockStyle.Left;
            lateralPanel.Location = new Point(0, 0);
            lateralPanel.Name = "lateralPanel";
            lateralPanel.Size = new Size(192, 541);
            lateralPanel.TabIndex = 0;
            // 
            // mainPanel
            // 
            mainPanel.BackColor = Color.Azure;
            mainPanel.Controls.Add(button1);
            mainPanel.Controls.Add(upperPanel);
            mainPanel.Dock = DockStyle.Fill;
            mainPanel.Location = new Point(195, 0);
            mainPanel.Name = "mainPanel";
            mainPanel.Size = new Size(807, 541);
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
            upperPanel.Size = new Size(807, 97);
            upperPanel.TabIndex = 0;
            // 
            // mainMenu
            // 
            mainMenu.Items.AddRange(new ToolStripItem[] { fileToolStripMenuItem });
            mainMenu.Location = new Point(0, 0);
            mainMenu.Name = "mainMenu";
            mainMenu.Size = new Size(1002, 24);
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
            basePanel.Size = new Size(1002, 541);
            basePanel.TabIndex = 1;
            // 
            // splitter1
            // 
            splitter1.Location = new Point(192, 0);
            splitter1.Name = "splitter1";
            splitter1.Size = new Size(3, 541);
            splitter1.TabIndex = 3;
            splitter1.TabStop = false;
            // 
            // MainForm
            // 
            AutoScaleDimensions = new SizeF(7F, 15F);
            AutoScaleMode = AutoScaleMode.Font;
            ClientSize = new Size(1002, 565);
            Controls.Add(basePanel);
            Controls.Add(mainMenu);
            DoubleBuffered = true;
            Name = "MainForm";
            StartPosition = FormStartPosition.CenterScreen;
            Text = "PDFix";
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
    }
}
